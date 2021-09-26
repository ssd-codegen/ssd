pub trait MapVec<A, B>
    where A: Into<B>,
{
    fn map_vec(self) -> Vec<B>;
}

impl<A, B> MapVec<A, B> for Vec<A>
    where A: Into<B>,
{
    fn map_vec(self) -> Vec<B> {
        self.into_iter().map(Into::into).collect()
    }
}
