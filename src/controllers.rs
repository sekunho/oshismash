pub mod assets;

use maud::Markup;
use crate::views;

/// Main page for the smash or pass
pub async fn root() -> Markup {
    views::root::render("Oshi Smash: Smash or Pass Your Oshis!", views::vote::render())
}
