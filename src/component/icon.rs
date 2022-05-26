use maud::{html, Markup, PreEscaped};

pub fn heart() -> Markup {
    html! {
        (PreEscaped("
            <svg xmlns=\"http://www.w3.org/2000/svg\" class=\"\" viewBox=\"0 0 20 20\" fill=\"currentColor\">
              <path fill-rule=\"evenodd\" d=\"M3.172 5.172a4 4 0 015.656 0L10 6.343l1.172-1.171a4 4 0 115.656 5.656L10 17.657l-6.828-6.829a4 4 0 010-5.656z\" clip-rule=\"evenodd\" />
            </svg>
        "))
    }
}

pub fn x() -> Markup {
    html! {
        (PreEscaped("
            <svg xmlns=\"http://www.w3.org/2000/svg\" class=\"h-6 w-6\" fill=\"none\" viewBox=\"0 0 24 24\" stroke=\"currentColor\" stroke-width=\"2\">
              <path stroke-linecap=\"round\" stroke-linejoin=\"round\" d=\"M6 18L18 6M6 6l12 12\" />
            </svg>
        "))
    }
}

pub fn chevron_up() -> Markup {
    html! {
        (PreEscaped("
<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"h-6 w-6\" fill=\"none\" viewBox=\"0 0 24 24\" stroke=\"currentColor\" stroke-width=\"2\">
  <path stroke-linecap=\"round\" stroke-linejoin=\"round\" d=\"M5 15l7-7 7 7\" />
</svg>
                    "))
    }
}

pub fn chevron_down() -> Markup {
    html! {
        (PreEscaped("
<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"h-6 w-6\" fill=\"none\" viewBox=\"0 0 24 24\" stroke=\"currentColor\" stroke-width=\"2\">
  <path stroke-linecap=\"round\" stroke-linejoin=\"round\" d=\"M19 9l-7 7-7-7\" />
</svg>
                    "))
    }
}

pub fn chevron_left() -> Markup {
    html! {
        (PreEscaped("
<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"h-6 w-6\" fill=\"none\" viewBox=\"0 0 24 24\" stroke=\"currentColor\" stroke-width=\"2\">
  <path stroke-linecap=\"round\" stroke-linejoin=\"round\" d=\"M15 19l-7-7 7-7\" />
</svg>
                    "))
    }
}

pub fn chevron_right() -> Markup {
    html! {
        (PreEscaped("
<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"h-6 w-6\" fill=\"none\" viewBox=\"0 0 24 24\" stroke=\"currentColor\" stroke-width=\"2\">
  <path stroke-linecap=\"round\" stroke-linejoin=\"round\" d=\"M9 5l7 7-7 7\" />
</svg>
                    "))
    }
}
