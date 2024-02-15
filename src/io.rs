use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader};

use crate::source::{Chunk, CodeChunk};

pub trait InputFile {
    async fn read_chunk(&mut self) -> Result<Option<Chunk>, tokio::io::Error>;
}

pub trait OutputFile {
    async fn write_chunk(&mut self, chunk: &Chunk) -> Result<(), tokio::io::Error>;
}

pub struct InputTypstFile<R: AsyncRead + Unpin> {
    buffer: BufReader<R>,
}

impl<R: AsyncRead + Unpin> InputTypstFile<R> {
    pub fn new(reader: R) -> Self {
        InputTypstFile {
            buffer: BufReader::new(reader),
        }
    }
}

impl<R: AsyncRead + Unpin> InputFile for InputTypstFile<R> {
    async fn read_chunk(&mut self) -> Result<Option<Chunk>, tokio::io::Error> {
        loop {
            let mut line = String::new();
            let mut code = String::new();
            let mut options = Vec::new();

            let r = self.buffer.read_line(&mut line).await?;
            if r == 0 {
                return Ok(None);
            }

            if line.trim().starts_with("```") {
                let lang = line.trim().strip_prefix("```").unwrap().trim();
                // read options
                let mut reading_options = true;
                loop {
                    let mut line = String::new();
                    let r = self.buffer.read_line(&mut line).await?;
                    if r == 0 {
                        return Err(tokio::io::Error::other("unexpected EOF"));
                    }
                    if reading_options && line.trim().starts_with("#|") {
                        let kv = line
                            .trim()
                            .strip_prefix("#|")
                            .unwrap()
                            .splitn(2, ':')
                            .map(str::trim)
                            .collect::<Vec<_>>();
                        options.push((kv[0].to_string(), kv[1].to_string()));
                    } else {
                        reading_options = false;
                        if line.trim().starts_with("```") {
                            break;
                        }
                        code.push_str(&line);
                    }
                }
                return Ok(Some(Chunk::Code(CodeChunk {
                    lang: lang.into(),
                    options: options.into_iter().collect(),
                    code: code.into(),
                })));
            } else {
                return Ok(Some(Chunk::Verbatim(line)));
            }
        }
    }
}

pub struct OutputTypstFile<W: tokio::io::AsyncWrite + Unpin> {
    writer: W,
}

impl<W: tokio::io::AsyncWrite + Unpin> OutputTypstFile<W> {
    pub fn new(writer: W) -> Self {
        OutputTypstFile { writer }
    }
}

impl<W: tokio::io::AsyncWrite + Unpin> OutputFile for OutputTypstFile<W> {
    async fn write_chunk(&mut self, chunk: &Chunk) -> Result<(), tokio::io::Error> {
        match chunk {
            Chunk::Verbatim(s) => {
                self.writer.write_all(s.as_bytes()).await?;
            }
            Chunk::Code(CodeChunk {
                lang,
                options: _,
                code,
            }) => {
                self.writer
                    .write_all(format!("#src[\n```{}\n", lang).as_bytes())
                    .await?;
                self.writer.write_all(code.as_bytes()).await?;
                self.writer.write_all("```\n]\n".as_bytes()).await?;
            }
            Chunk::Output(o) => {
                self.writer
                    .write_all(format!("```\n{}\n```\n", o.data).as_bytes())
                    .await?;
            }
            Chunk::Error(e) => {
                self.writer
                    .write_all(format!("#emoji.crossmark {}\n", e).as_bytes())
                    .await?;
            }
            Chunk::Graphics(g) => match g.ty {
                crate::source::GraphicsType::Png => {
                    self.writer
                        .write_all(
                            format!(
                                "#image.decode(bytes(({})))\n",
                                g.data
                                    .iter()
                                    .map(|e| e.to_string())
                                    .collect::<Vec<_>>()
                                    .join(",")
                            )
                            .as_bytes(),
                        )
                        .await?;
                }
            },
            _ => unimplemented!("not implemented"),
        }
        Ok(())
    }
}
