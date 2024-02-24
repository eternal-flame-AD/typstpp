use rand::{rngs::ThreadRng, Rng};
use std::process::Stdio;
use tokio::process::Command;
use typstpp_backend::{Backend, Input};

pub struct HsBackend;

impl HsBackend {
    pub fn new_cookie(&self, rng: &mut ThreadRng) -> String {
        let bytes = std::iter::repeat(())
            .map(|()| rng.sample(rand::distributions::Alphanumeric))
            .take(16)
            .collect();
        String::from_utf8(bytes).unwrap()
    }
}

pub struct HsOptions {
    echo: bool,
    eval: bool,
}

impl From<std::collections::HashMap<String, String>> for HsOptions {
    fn from(m: std::collections::HashMap<String, String>) -> Self {
        HsOptions {
            echo: m
                .get("echo")
                .map(|s| s == "true" || s == "1" || s == "yes")
                .unwrap_or(true),
            eval: m
                .get("eval")
                .map(|s| s == "true" || s == "1" || s == "yes")
                .unwrap_or(true),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Eval error: {0}")]
    EvalError(String),
}

#[async_trait::async_trait]
impl Backend for HsBackend {
    type GlobalOptions = ();
    type Options = HsOptions;
    type Output = String;
    type Error = Error;

    async fn new<'a>(
        global_options: Self::GlobalOptions,
    ) -> Result<Self, typstpp_backend::Error<Self::Error>>
    where
        Self: Sized,
    {
        Ok(HsBackend)
    }

    async fn compile<'a>(
        &mut self,
        input: Vec<Input<'a, Self::Options>>,
    ) -> Result<Vec<Vec<typstpp_backend::Output<Self::Output>>>, typstpp_backend::Error<Self::Error>>
    {
        let cookies = (0..input.len())
            .map(|_| self.new_cookie(&mut rand::thread_rng()))
            .collect::<Vec<_>>();
        let mut child = Command::new("ghc");
        child
            .stdout(Stdio::piped())
            .stdin(Stdio::null())
            .stderr(Stdio::piped());
        for (i, input) in input.iter().enumerate() {
            if input.options.eval {
                for line in input.source.lines() {
                    child.arg("-e").arg(line);
                }
            }
            child.arg("-e").arg(format!("putStrLn \"{}\"", cookies[i]));
        }
        let output = child.output().await.map_err(|e| {
            typstpp_backend::Error::BackendError(Error::EvalError(format!("{}", e)))
        })?;
        let mut stdout = String::from_utf8(output.stdout).map_err(|e| {
            typstpp_backend::Error::BackendError(Error::EvalError(format!("{}", e)))
        })?;
        let stderr = String::from_utf8(output.stderr).map_err(|e| {
            typstpp_backend::Error::BackendError(Error::EvalError(format!("{}", e)))
        })?;
        let mut outputs = vec![];
        for (i, cookie) in cookies.iter().enumerate() {
            let breakpoint = stdout
                .find(cookie)
                .ok_or(typstpp_backend::Error::BackendError(Error::EvalError(
                    format!("Cookie not found, stderr: {}", stderr),
                )))?;
            let mut chunk_output = vec![];
            if input[0].options.echo {
                chunk_output.push(typstpp_backend::Output {
                    data: input[i].source.to_string(),
                    ty: typstpp_backend::OutputType::Code,
                });
            }
            if !stderr.is_empty() {
                chunk_output.push(typstpp_backend::Output {
                    data: stderr.clone(),
                    ty: typstpp_backend::OutputType::Error,
                });
            }
            chunk_output.push(typstpp_backend::Output {
                data: stdout.drain(0..breakpoint).collect(),
                ty: typstpp_backend::OutputType::Output,
            });
            stdout.drain(0..cookie.len());
            outputs.push(chunk_output);
        }
        Ok(outputs)
    }

    async fn reset(&mut self) -> Result<(), typstpp_backend::Error<Self::Error>> {
        Ok(())
    }

    async fn close(mut self) -> Result<(), typstpp_backend::Error<Self::Error>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hs_backend() {
        let mut backend = HsBackend::new(()).await.unwrap();
        let input = vec![Input {
            source: "putStrLn \"Hello, world!\"",
            options: HsOptions {
                echo: true,
                eval: true,
            },
        }];
        let outputs = backend.compile(input).await.unwrap();
        assert_eq!(
            outputs,
            vec![vec![typstpp_backend::Output {
                data: "Hello, world!\n".to_string(),
                ty: typstpp_backend::OutputType::Output
            }]]
        );
    }
}
