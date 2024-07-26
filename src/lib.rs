use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
};

use crate::{io::InputFile, io::OutputFile};
use source::CodeChunk;
use tokio::{
    fs,
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
};
use typstpp_backend::{Backend, Input};
mod io;
mod source;

#[derive(Debug)]
pub struct CodeOutput<FO: Display> {
    pub errors: Vec<String>,
    pub outputs: Vec<typstpp_backend::Output<FO>>,
}

pub struct LanguageDriver<O, FO, B: typstpp_backend::Backend> {
    backend: B,
    _phantom: std::marker::PhantomData<(O, FO)>,
}

impl<O, FO, B: typstpp_backend::Backend> LanguageDriver<O, FO, B> {
    pub fn new(backend: B) -> Self {
        LanguageDriver {
            backend,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
pub trait Preprocess<FO: Display> {
    async fn preprocess(&mut self, input: &Vec<&CodeChunk>) -> Vec<CodeOutput<FO>>;
}

#[async_trait::async_trait]
impl<O, FO, B> Preprocess<FO> for LanguageDriver<O, FO, B>
where
    O: Send,
    FO: Display + Send,
    B: typstpp_backend::Backend + Send,
    <B as Backend>::Options: From<HashMap<String, String>>,
    typstpp_backend::Output<FO>: From<typstpp_backend::Output<<B as Backend>::Output>>,
{
    async fn preprocess(&mut self, input: &Vec<&CodeChunk>) -> Vec<CodeOutput<FO>> {
        let input = input
            .iter()
            .map(|c| Input {
                source: c.code.as_ref(),
                options: c.options.clone().into(),
            })
            .collect::<Vec<_>>();

        let result = self.backend.compile(input).await;

        match result {
            Ok(outputs) => outputs
                .into_iter()
                .map(|o| CodeOutput {
                    errors: vec![],
                    outputs: o.into_iter().map(|o| o.into()).collect(),
                })
                .collect(),
            Err(e) => vec![CodeOutput {
                errors: vec![format!("{}", e)],
                outputs: vec![],
            }],
        }
    }
}

pub struct DocumentDriver<FO> {
    backends: HashMap<String, Box<dyn Preprocess<FO>>>,
}

impl<FO> Default for DocumentDriver<FO>
where
    FO: Display,
 {
    fn default() -> Self {
        Self::new()
    }
}

impl<FO> DocumentDriver<FO>
where
    FO: Display,
{
    pub fn new() -> Self {
        DocumentDriver {
            backends: HashMap::new(),
        }
    }
    pub fn add_backend(&mut self, name: String, backend: Box<dyn Preprocess<FO>>) {
        self.backends.insert(name, backend);
    }
}

#[derive(Debug)]
pub enum Error {
    IO(tokio::io::Error),
    RuntimeError(String),
}

impl From<tokio::io::Error> for Error {
    fn from(e: tokio::io::Error) -> Self {
        Error::IO(e)
    }
}

pub async fn preprocess_typst<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
    reader: R,
    mut writer: W,
) -> Result<(), Error> {
    let mut driver: DocumentDriver<String> = DocumentDriver::new();
    #[cfg(feature = "r")]
    driver.add_backend(
        "r".to_string(),
        Box::new(LanguageDriver::<typstpp_r::ROptions, _, _>::new(
            typstpp_r::RBackend::new(typstpp_r::RGlobalOptions::default())
                .await
                .unwrap(),
        )),
    );
    #[cfg(feature = "hs")]
    driver.add_backend(
        "hs".to_string(),
        Box::new(LanguageDriver::<typstpp_hs::HsOptions, _, _>::new(
            typstpp_hs::HsBackend::new(()).await.unwrap(),
        )),
    );
    writer.write_all(include_bytes!("prelude.typ")).await?;

    let mut input = io::InputTypstFile::new(reader);
    let mut output = io::OutputTypstFile::new(writer);

    let mut chunks = Vec::new();
    while let Some(chunk) = input.read_chunk().await? {
        chunks.push(chunk);
    }
    let code_chunks = chunks.iter_mut().filter_map(|c| match c {
        source::Chunk::Code(c) => Some(c),
        _ => None,
    });

    let mut code_chunks_by_lang = HashMap::new();
    for c in code_chunks {
        if let Some(file) = c.options.get("file") {
            c.code = fs::read_to_string(file).await?;
        }
        code_chunks_by_lang
            .entry(c.lang.clone())
            .or_insert_with(Vec::new)
            .push(&*c);
    }
    let mut code_outputs_by_lang = HashMap::new();
    for (lang, chunks) in code_chunks_by_lang {
        if let Some(backend) = driver.backends.get_mut(&lang) {
            let result = backend.preprocess(&chunks).await;
            code_outputs_by_lang.insert(lang.clone(), VecDeque::from(result));
        } else {
            code_outputs_by_lang.insert(
                lang.clone(),
                chunks
                    .iter()
                    .map(|c| CodeOutput {
                        errors: vec![],
                        outputs: vec![typstpp_backend::Output {
                            data: c.code.clone(),
                            ty: typstpp_backend::OutputType::Code,
                        }],
                    })
                    .collect::<VecDeque<_>>(),
            );
        }
    }
    for chunk in chunks {
        match chunk {
            source::Chunk::Verbatim(s) => output.write_chunk(&source::Chunk::Verbatim(s)).await?,
            source::Chunk::Code(c) => {
                let lang = c.lang.clone();
                let outputs = code_outputs_by_lang
                    .get_mut(&lang)
                    .and_then(|o| o.pop_front())
                    .unwrap_or_else(|| CodeOutput {
                        errors: vec![],
                        outputs: vec![],
                    });
                for e in outputs.errors {
                    output.write_chunk(&source::Chunk::Error(e)).await?;
                }
                for o in outputs.outputs {
                    match o.ty {
                        typstpp_backend::OutputType::Typst => {
                            output
                                .write_chunk(&source::Chunk::Verbatim(o.data.to_string()))
                                .await?
                        }
                        typstpp_backend::OutputType::Code => {
                            output
                                .write_chunk(&source::Chunk::Code(CodeChunk {
                                    lang: c.lang.clone(),
                                    options: Default::default(),
                                    code: o.data,
                                }))
                                .await?;
                        }
                        typstpp_backend::OutputType::Output => {
                            output.write_chunk(&source::Chunk::Output(o)).await?
                        }
                        typstpp_backend::OutputType::Message => {
                            output
                                .write_chunk(&source::Chunk::Message(o.data.to_string()))
                                .await?
                        }
                        typstpp_backend::OutputType::Error => {
                            output
                                .write_chunk(&source::Chunk::Error(o.data.to_string()))
                                .await?
                        }
                    }
                }
            }
            source::Chunk::Output(o) => output.write_chunk(&source::Chunk::Output(o)).await?,
            source::Chunk::Message(m) => output.write_chunk(&source::Chunk::Message(m)).await?,
            source::Chunk::Error(e) => output.write_chunk(&source::Chunk::Error(e)).await?,
            source::Chunk::Graphics(g) => output.write_chunk(&source::Chunk::Graphics(g)).await?,
        }
    }

    Ok(())
}
