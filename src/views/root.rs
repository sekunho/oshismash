use maud::{Markup, DOCTYPE, html};

pub fn render(title: &str, content: Markup) -> Markup {
    html! {
        (header(title))
        (content)
    }
}

fn header(title: &str) -> Markup {
    html! {
        (DOCTYPE)
        meta charset="utf-8";

        link rel="stylesheet" href="/assets/app.css";
        link rel="preconnect" href="https://fonts.googleapis.com";
        link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
        link href="https://fonts.googleapis.com/css2?family=Vollkorn:wght@600&display=swap&text=oshima" rel="stylesheet";
        meta name="viewport" content="width=device-width, initial-scale=1.0";

        title { (title) }
    }
}
