// Em src/cli/route.rs ou no mesmo arquivo se estiver tudo junto

pub struct Route {
    pub path: &'static str,
    pub handler_name: &'static str,
}
