use maud::{Markup, html};

use crate::oshismash;

pub async fn show(

) -> Result<Markup, oshismash::Error> {
    let template = html! {
        "Penis"
    };

    Ok(template)
}
