pub trait MapVec<Old, New>
where
    Old: Into<New>,
{
    fn map_vec(self) -> Vec<New>;
}

impl<Old, New> MapVec<Old, New> for Vec<Old>
where
    Old: Into<New>,
{
    fn map_vec(self) -> Vec<New> {
        self.into_iter().map(Into::into).collect()
    }
}
