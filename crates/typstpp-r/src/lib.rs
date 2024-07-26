use libR_sys::{
    cetype_t_CE_UTF8, setup_Rmainloop, R_CStackLimit, R_CleanTempDir, R_FindNamespace, R_GlobalEnv,
    R_RunExitFinalizers, R_tryEval, Rf_ScalarString, Rf_findFun, Rf_initialize_R, Rf_install,
    Rf_lang2, Rf_mkCharLenCE, Rf_mkString, Rf_protect, Rf_translateCharUTF8, Rf_unprotect_ptr, CDR,
    SET_TAG, SEXPREC, STRING_ELT,
};
use rand::Rng;
use std::{collections::HashMap, ffi::CStr, ops::Deref};
use table::transform_tables;
use tokio::{process::Command, sync::OnceCell};
mod io;
mod table;

use typstpp_backend::Backend;

struct RObj(*mut SEXPREC);

impl RObj {
    fn new(ptr: *mut SEXPREC) -> Self {
        unsafe {
            Rf_protect(ptr);
        }
        RObj(ptr)
    }
}

impl Drop for RObj {
    fn drop(&mut self) {
        unsafe {
            Rf_unprotect_ptr(self.0 as *mut _);
        }
    }
}

impl From<*mut SEXPREC> for RObj {
    fn from(ptr: *mut SEXPREC) -> Self {
        RObj::new(ptr)
    }
}

impl Deref for RObj {
    type Target = *mut SEXPREC;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RBackend;

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("R error: {0}")]
    RError(&'static str),
}

fn reindent(input: &str, output_from: String) -> String {
    let first_line = match input.lines().next() {
        Some(l) => l,
        None => return output_from,
    };
    let indent = first_line
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect::<String>();
    let mut output = String::new();
    for line in output_from.lines() {
        output.push_str(&indent);
        output.push_str(line);
        output.push('\n');
    }
    output
}

impl RBackend {
    pub fn new_cookie(&self) -> String {
        let mut rng = rand::thread_rng();
        let bytes = std::iter::repeat(())
            .map(|()| rng.sample(rand::distributions::Alphanumeric))
            .take(10)
            .collect();
        String::from_utf8(bytes).unwrap()
    }

    async fn pass<'a>(
        &mut self,
        input: typstpp_backend::Input<'a, <Self as Backend>::Options>,
    ) -> Result<
        Vec<typstpp_backend::Output<<Self as Backend>::Output>>,
        typstpp_backend::Error<<Self as Backend>::Error>,
    > {
        let knitr_char = unsafe { Rf_mkString("knitr\0".as_ptr() as *const i8) };
        let knitr = unsafe { R_FindNamespace(knitr_char) };
        let knitr = RObj::from(knitr);
        let knit = unsafe { Rf_findFun(Rf_install("knit\0".as_ptr() as *const i8), *knitr) };
        let knit = RObj::from(knit);
        let source_wrapped = format!(
            "```{{r {}}}\n{}\n```",
            {
                [
                    input
                        .options
                        .echo
                        .map(|b| format!("echo={}", if b { "TRUE" } else { "FALSE" })),
                    input
                        .options
                        .eval
                        .map(|b| format!("eval={}", if b { "TRUE" } else { "FALSE" })),
                    input
                        .options
                        .error
                        .map(|b| format!("error={}", if b { "TRUE" } else { "FALSE" })),
                    input
                        .options
                        .include
                        .map(|b| format!("include={}", if b { "TRUE" } else { "FALSE" })),
                    input
                        .options
                        .message
                        .map(|b| format!("message={}", if b { "TRUE" } else { "FALSE" })),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(", ")
            },
            input.source
        );
        let code = unsafe {
            Rf_mkCharLenCE(
                source_wrapped.as_ptr() as *const i8,
                i32::try_from(source_wrapped.len()).unwrap(),
                cetype_t_CE_UTF8,
            )
        };
        let code = RObj::from(code);
        let code_str = unsafe { Rf_ScalarString(*code) };
        let code_str = RObj::from(code_str);
        let call = unsafe { Rf_lang2(*knit, *code_str) };
        let call = RObj::from(call);
        unsafe {
            SET_TAG(CDR(*call), Rf_install("text\0".as_ptr() as *const i8));
        }
        let mut error_occurred = 0;

        let result = unsafe { R_tryEval(*call, R_GlobalEnv, &mut error_occurred) };
        if error_occurred != 0 {
            return Err(typstpp_backend::Error::BackendError(Error::RError(
                "Error occurred",
            )));
        }
        let result = unsafe {
            let result = STRING_ELT(result, 0);
            let result = Rf_translateCharUTF8(result);
            String::from_utf8(CStr::from_ptr(result).to_bytes().to_vec()).unwrap()
        };
        let result = transform_tables(&result);
        let result = result.replace("```\n]\n#src[\n```r\n", "");
        let result = reindent(input.source, result);
        Ok(vec![typstpp_backend::Output {
            data: result,
            ty: typstpp_backend::OutputType::Typst,
        }])
    }
}

#[derive(Debug, Clone, Default)]
pub struct ROptions {
    echo: Option<bool>,
    eval: Option<bool>,
    error: Option<bool>,
    include: Option<bool>,
    message: Option<bool>,
}

impl From<HashMap<String, String>> for ROptions {
    fn from(m: HashMap<String, String>) -> Self {
        ROptions {
            echo: m.get("echo").map(|s| s.parse().unwrap()),
            eval: m.get("eval").map(|s| s.parse().unwrap()),
            error: m.get("error").map(|s| s.parse().unwrap()),
            include: m.get("include").map(|s| s.parse().unwrap()),
            message: m.get("message").map(|s| s.parse().unwrap()),
        }
    }
}

static mut R_INITIALIZED: OnceCell<Result<(), typstpp_backend::Error<Error>>> =
    OnceCell::const_new();

#[async_trait::async_trait]
impl Backend for RBackend {
    type GlobalOptions = ();
    type Options = ROptions;
    type Output = String;
    type Error = Error;

    async fn new<'a>(
        global_options: Self::GlobalOptions,
    ) -> Result<Self, typstpp_backend::Error<Self::Error>>
    where
        Self: Sized,
    {
        unsafe {
            R_INITIALIZED
                .get_or_init(|| async {
                    if !std::env::var("R_HOME").is_ok() {
                        let out = Command::new("R")
                            .arg("-s")
                            .arg("-e")
                            .arg("cat(normalizePath(R.home()))")
                            .output()
                            .await;
                        match out {
                            Ok(out) => {
                                let home = String::from_utf8(out.stdout).unwrap();
                                std::env::set_var("R_HOME", home.trim());
                            }
                            Err(_) => {
                                return Err(typstpp_backend::Error::BackendError(Error::RError(
                                    "Failed to find R_HOME",
                                )))
                            }
                        }
                    }
                    if Rf_initialize_R(
                        3,
                        ["R\0".as_ptr(), "--slave\0".as_ptr(), "--silent\0".as_ptr()].as_ptr()
                            as *mut *mut i8,
                    ) != 0
                    {
                        return Err(typstpp_backend::Error::BackendError(Error::RError(
                            "Failed to initialize R",
                        )));
                    }
                    R_CStackLimit = usize::MAX;
                    setup_Rmainloop();
                    let prelude = include_str!("prelude.R");
                    let prelude = Rf_mkCharLenCE(
                        prelude.as_ptr() as *const i8,
                        i32::try_from(prelude.len()).unwrap(),
                        cetype_t_CE_UTF8,
                    );
                    let prelude = RObj::from(prelude);
                    let prelude_str = Rf_ScalarString(*prelude);
                    let prelude_str = RObj::from(prelude_str);
                    let parse_call =
                        Rf_lang2(Rf_install("parse\0".as_ptr() as *const i8), *prelude_str);
                    let parse_call = RObj::from(parse_call);
                    SET_TAG(CDR(*parse_call), Rf_install("text\0".as_ptr() as *const i8));
                    let mut error_occurred = 0;
                    let prelude_expr = R_tryEval(*parse_call, R_GlobalEnv, &mut error_occurred);
                    if error_occurred != 0 {
                        return Err(typstpp_backend::Error::BackendError(Error::RError(
                            "Failed to parse prelude",
                        )));
                    }
                    let prelude_expr = RObj::from(prelude_expr);

                    let call = Rf_lang2(Rf_install("eval\0".as_ptr() as *const i8), *prelude_expr);
                    let call = RObj::from(call);
                    let mut error_occurred = 0;
                    R_tryEval(*call, R_GlobalEnv, &mut error_occurred);
                    if error_occurred != 0 {
                        return Err(typstpp_backend::Error::BackendError(Error::RError(
                            "Failed to evaluate prelude",
                        )));
                    }
                    Ok(())
                })
                .await
                .as_ref()
                .map_err(|e| e.clone())
                .map(|_| RBackend {})
        }
    }

    async fn compile<'a>(
        &mut self,
        input: Vec<typstpp_backend::Input<'a, Self::Options>>,
    ) -> Result<Vec<Vec<typstpp_backend::Output<Self::Output>>>, typstpp_backend::Error<Self::Error>>
    {
        let mut outputs = Vec::new();
        for i in input {
            outputs.push(self.pass(i).await?);
        }
        Ok(outputs)
    }

    async fn reset(&mut self) -> Result<(), typstpp_backend::Error<Self::Error>> {
        let ls = unsafe { Rf_lang2(Rf_install("ls\0".as_ptr() as *const i8), R_GlobalEnv) };
        let ls = RObj::from(ls);
        let mut error_occurred = 0;
        let result = unsafe { R_tryEval(*ls, R_GlobalEnv, &mut error_occurred) };
        if error_occurred != 0 {
            return Err(typstpp_backend::Error::BackendError(Error::RError(
                "Error occurred",
            )));
        }
        let objs = RObj::from(result);
        let rm_call = unsafe { Rf_lang2(Rf_install("rm\0".as_ptr() as *const i8), *objs) };
        let rm_call = RObj::from(rm_call);
        unsafe { SET_TAG(CDR(*rm_call), Rf_install("list\0".as_ptr() as *const i8)) };
        let mut error_occurred = 0;
        unsafe { R_tryEval(*rm_call, R_GlobalEnv, &mut error_occurred) };
        if error_occurred != 0 {
            return Err(typstpp_backend::Error::BackendError(Error::RError(
                "Error occurred",
            )));
        }
        Ok(())
    }

    async fn close(mut self) -> Result<(), typstpp_backend::Error<Self::Error>> {
        unsafe {
            R_RunExitFinalizers();
            R_CleanTempDir();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typstpp_backend::Backend;
    #[tokio::test]
    async fn test_r_backend() {
        let mut backend = RBackend::new(()).await.unwrap();
        let result = backend
            .pass(typstpp_backend::Input {
                source: "print('hello')".into(),
                options: ROptions::default(),
            })
            .await
            .unwrap();
        assert_eq!(
            result,
            vec![typstpp_backend::Output {
                data: "```r\nprint('hello')\n```\n```\n## [1] \"hello\"\n\n```\n\n".to_string(),
                ty: typstpp_backend::OutputType::Typst,
            }]
        );
        let result = backend
            .pass(typstpp_backend::Input {
                source: "a <- 1+1\nprint(a)".into(),
                options: ROptions::default(),
            })
            .await
            .unwrap();
        assert_eq!(
            result,
            vec![typstpp_backend::Output {
                data: "```r\na <- 1+1\n```\n```r\nprint(a)\n```\n```\n## [1] 2\n\n```\n\n"
                    .to_string(),
                ty: typstpp_backend::OutputType::Typst,
            }]
        );

        let result = backend
            .pass(typstpp_backend::Input {
                source: "a <- 1".into(),
                options: ROptions::default(),
            })
            .await
            .unwrap();
        assert_eq!(
            result,
            vec![typstpp_backend::Output {
                data: "```r\na <- 1\n```\n\n".to_string(),
                ty: typstpp_backend::OutputType::Typst,
            }]
        );
        let result = backend
            .pass(typstpp_backend::Input {
                source: "print(a)".into(),
                options: ROptions::default(),
            })
            .await
            .unwrap();
        assert_eq!(
            result,
            vec![typstpp_backend::Output {
                data: "```r\nprint(a)\n```\n```\n## [1] 1\n\n```\n\n".to_string(),
                ty: typstpp_backend::OutputType::Typst,
            }]
        );

        backend.reset().await.unwrap();

        let result = backend
            .pass(typstpp_backend::Input {
                source: "print(a)".into(),
                options: ROptions::default(),
            })
            .await
            .unwrap();

        assert!(result[0].data.contains("Error"))
    }
}
