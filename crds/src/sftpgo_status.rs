pub trait SftpgoStatus {
    fn get_last_name(&self) -> &str;
    fn set_last_name(&mut self, name: &str);

    fn get_id(&self) -> Option<i32>;
    fn set_id(&mut self, id: Option<i32>);


}