use yew_router::Routable;
pub use yew_router_extra_macro::TitledRoutable;

pub trait TitledRoutable: Routable {
    fn title(&self) -> String;
}
