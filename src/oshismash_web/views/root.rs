use maud::{html, Markup, DOCTYPE};

use crate::oshismash_web::components::icon;

pub fn render(title: &str, content: Markup) -> Markup {
    html! {
        (header(title))

        body class="bg-gray-100 dark:bg-su-dark-bg-1 h-screen flex flex-col" {
            main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 w-full flex flex-col flex-1 py-4 md:py-8" {
                div class="flex items-center justify-between md:justify-center mb-4 md:mb-8" {
                    (logo())

                    nav class="dark:text-su-dark-fg-1 flex space-x-2 md:hidden" {
                        a target="_blank" href="https://ko-fi.com/sekun" {
                            (icon::money())
                        }

                        a target="_blank" href="https://twitter.com/sekunho_" {
                            (icon::twitter())
                        }

                        a target="_blank" href="https://youtube.com/sekunho" {
                            (icon::youtube())
                        }

                        a target="_blank" href="https://github.com/sekunho/oshismash" {
                            (icon::github())
                        }
                    }
                }

                (content)
            }
        }

        (footer())
    }
}

fn logo() -> Markup {
    html! {
        a href="/" class="font-serif mb-auto dark:text-su-dark-fg-1 font-semibold uppercase text-2xl md:text-4xl text-center" { ("Oshi Smash") }
    }
}

fn header(title: &str) -> Markup {
    html! {
        (DOCTYPE)
        meta charset="utf-8";

        link rel="stylesheet" href="/assets/app.css";
        link rel="preconnect" href="https://fonts.googleapis.com";
        link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
        link href="https://fonts.googleapis.com/css2?family=Vollkorn:wght@600&display=swap&text=OSHIMA" rel="stylesheet";

        link rel="apple-touch-icon" sizes="180x180" href="/assets/apple-touch-icon.png";
        link rel="icon" type="image/png" sizes="32x32" href="/assets/favicon-32x32.png";
        link rel="icon" type="image/png" sizes="16x16" href="/assets/favicon-16x16.png";
        link rel="manifest" href="/assets/site.webmanifest";

        meta name="viewport" content="width=device-width, initial-scale=1.0";

        title { (title) }
    }
}

fn footer() -> Markup {
    html! {
        footer class="hidden md:block bg-gray-100 dark:bg-su-dark-bg-1" {
            div class="max-w-7xl mx-auto pb-6 pt-2 px-4 sm:px-6 flex flex-col-reverse items-center md:flex-row md:items-center md:justify-between lg:px-8 text-su-fg-1 dark:text-su-dark-fg-1" {
                span class="hidden md:block mt-4 md:mt-0" {
                    "Made by "
                    a href="https://sekun.dev" target="_blank" class="underline decoration-wavy decoration-red-500" {
                        "SEKUN"
                    }
                    " © 2022"
                }

                nav class="space-y-1 sm:space-y-0 space-x-5 flex items-end" {
                    a href="/" {
                        "Home"
                    }

                    a target="_blank" href="https://ko-fi.com/sekun" {
                        (icon::money())
                    }

                    a target="_blank" href="https://twitter.com/sekunho_" {
                        (icon::twitter())
                    }

                    a target="_blank" href="https://youtube.com/sekunho" {
                        (icon::youtube())
                    }

                    a target="_blank" href="https://github.com/sekunho/oshismash" {
                        (icon::github())
                    }
                }
            }
        }

    }
}
