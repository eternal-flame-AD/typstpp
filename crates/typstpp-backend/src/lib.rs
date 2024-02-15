use std::fmt::{Debug, Display};

pub struct Input<'a, O> {
    pub source: &'a str,
    pub options: O,
}

#[derive(Debug, Clone)]
pub struct Output<S: Display> {
    pub data: S,
    pub ty: OutputType,
}

impl<S> PartialEq for Output<S>
where
    S: PartialEq + Display,
{
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data && self.ty == other.ty
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputType {
    Typst,
    Code,
    Output,
    Message,
    Error,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error<E: Display> {
    #[error("Backend error: {0}")]
    BackendError(E),
}

#[async_trait::async_trait]
pub trait Backend {
    type GlobalOptions;
    type Options;
    type Output: Display;
    type Error: Display + Debug;
    async fn new<'a>(global_options: Self::GlobalOptions) -> Result<Self, Error<Self::Error>>
    where
        Self: Sized;
    async fn compile<'a>(
        &mut self,
        input: Vec<Input<'a, Self::Options>>,
    ) -> Result<Vec<Vec<Output<Self::Output>>>, Error<Self::Error>>;
    async fn reset(&mut self) -> Result<(), Error<Self::Error>>;
    async fn close(self) -> Result<(), Error<Self::Error>>;
}
