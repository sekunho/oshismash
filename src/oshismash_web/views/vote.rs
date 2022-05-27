use maud::{html, Markup};

use crate::{
    oshismash::vtubers::{Stack, VTuber},
    oshismash_web::components::icon,
};

pub fn render(stack: Stack) -> Markup {
    let current_vtuber = stack.get_current();

    html! {
        div class="flex flex-col justify-center items-center h-screen" {
            // Cards
            div class="relative w-full sm:w-2/3 lg:w-1/3 vh-60" {
                @if let Some(vtuber) = current_vtuber {
                    @if let Some(_) = vtuber.next {
                        div id="card" class="top-10 left-0 right-0 mx-auto absolute rounded-lg shadow-lg opacity-50 bg-su-bg-2 dark:bg-su-dark-bg-2 w-10/12 vh-60 mx-auto" {
                        }
                    }

                    div id="card" class="top-5 left-0 right-0 mx-auto absolute rounded-lg shadow-lg opacity-70 bg-su-bg-2 dark:bg-su-dark-bg-2 w-11/12 vh-60 mx-auto" {
                    }

                    (card(vtuber))
                } @else {
                    (last_card())
                }
            }

            @if let Some(vtuber) = current_vtuber {
                div class="flex mt-16 space-x-2.5" {
                    (pass(vtuber))
                    (smash(vtuber))
                }
            }

            @if let Some(stat) = stack.get_last_voted_stat() {
                span class="dark:text-su-dark-fg-1 mt-6" { (format!("What others voted for {}", stat.name)) }
                div class="flex justify-center space-x-2 w-full sm:w-2/3 md:w-1/3 mt-4 dark:text-su-dark-fg-1" {
                    div class="flex flex-col w-full items-end" {
                        span class="font-bold mb-2.5 text-right" { ("Passes") }
                        span
                            class="rounded-md h-8 bg-gradient-to-l from-red-500 to-pink-500"
                            style=(style_percentage(stat.passes, stat.smashes + stat.passes)) {

                            }

                        span class="font-bold text-lg text-right" { (stat.passes) }
                    }

                    figure class="flex-none w-24 aspect-square bg-su-dark-bg-2 rounded-md" {
                        img class="object-cover object-top h-full w-full" src=(stat.img.as_ref().unwrap());
                    }

                    div class="flex flex-col w-full" {
                        span class="font-bold mb-2.5" { ("Smashes") }
                        span
                            class="rounded-md h-8 bg-gradient-to-r from-cyan-500 to-blue-500"
                            style=(style_percentage(stat.smashes, stat.smashes + stat.passes)) {

                            }

                        span class="font-bold text-lg" { (stat.smashes) }
                    }
                }
            }
        }
    }
}

fn style_percentage(nominator: i64, denominator: i64) -> String {
    format!(
        "width: {}%;",
        (nominator as f64 / denominator as f64) * 100 as f64
    )
}

fn last_card() -> Markup {
    html! {
        div id="card" class="flex items-center justify-center absolute rounded-lg shadow-lg bg-su-bg-2 dark:bg-su-dark-bg-2 w-full vh-60 mx-auto" {
            span class="font-medium text-white text-2xl text-center" {
                ("You can touch grass now.")
            }
        }
    }
}

fn card(vtuber: &VTuber) -> Markup {
    html! {
        div id="card" class="absolute rounded-lg shadow-lg bg-su-bg-2 dark:bg-su-dark-bg-2 w-full vh-60 mx-auto" {
            figure class="h-full w-full rounded-lg relative" {
                img class="object-top object-cover h-full w-full rounded-lg" src=(vtuber.img);

                figcaption class="w-full left-0 bottom-0 rounded-b-lg absolute bg-gradient-to-t from-black p-4" {
                    div class="flex items-center space-x-2.5" {
                        h1 class="font-bold text-white text-3xl" {
                            (vtuber.name)
                        }

                        span class="text-white text-lg" {
                            (vtuber.org_name)
                        }
                    }

                    p class="text-lg text-white overflow-y-auto max-h-24" {
                        (vtuber.description)
                    }
                }


                div class="top-2 left-2 absolute space-y-2" {
                    (next_button(vtuber.next))
                    (prev_button(vtuber.prev))
                }
            }
        }
    }
}

// TODO: Use newtype
fn prev_button(vtuber_id: Option<i64>) -> Markup {
    html! {
        @match vtuber_id {
            Some(vtuber_id) =>  {
                form method="POST" action="/" {
                    input class="hidden" type="text" name="action" value="prev";
                    input class="hidden" type="text" name="vtuber_id" value=(vtuber_id);
                    button class="rounded-full h-6 w-6 dark:bg-su-dark-bg-1" {
                        p class="mx-auto h-5 w-5 dark:text-su-dark-fg-1 flex items-center justify-center" {
                            (icon::chevron_down())
                        }
                    }
                }

            }

            None => {
                button class="rounded-full h-6 w-6 dark:bg-su-dark-bg-1 opacity-70 cursor-not-allowed" disabled {
                    p class="mx-auto h-5 w-5 dark:text-su-dark-fg-1 flex items-center justify-center" {
                        (icon::chevron_down())
                    }
                }
            }
        }
    }
}

fn next_button(vtuber_id: Option<i64>) -> Markup {
    html! {
        @match vtuber_id {
            Some(vtuber_id) =>  {
                form method="POST" action="/" {
                    input class="hidden" type="text" name="action" value="next";
                    input class="hidden" type="text" name="vtuber_id" value=(vtuber_id);
                    button class="rounded-full h-6 w-6 dark:bg-su-dark-bg-1" {
                        p class="mx-auto h-5 w-5 dark:text-su-dark-fg-1 flex items-center justify-center" {
                            (icon::chevron_up())
                        }
                    }
                }

            }

            None => {
                button class="rounded-full h-6 w-6 dark:bg-su-dark-bg-1 opacity-70 cursor-not-allowed" disabled {
                    p class="mx-auto h-5 w-5 dark:text-su-dark-fg-1 flex items-center justify-center" {
                        (icon::chevron_up())
                    }
                }
            }
        }
    }
}

fn smash(vtuber: &VTuber) -> Markup {
    html! {
        form method="POST" action="/" {
            input class="hidden" type="text" name="action" value="smashed";
            input class="hidden" type="text" name="vtuber_id" value=(vtuber.id);

            button class="shadow-md rounded-full h-14 w-14 bg-gradient-to-t from-purple-500 to-pink-500" {
                p class="mx-auto h-8 w-8 text-white flex items-center justify-center" { (icon::heart()) }
            }
        }
    }
}

fn pass(vtuber: &VTuber) -> Markup {
    html! {
        form method="POST" action="/" {
            input class="hidden" type="text" name="action" value="passed";
            input class="hidden" type="text" name="vtuber_id" value=(vtuber.id);

            button class="shadow-md rounded-full h-14 w-14 bg-su-bg-2 dark:bg-su-dark-bg-2" {
                p class="mx-auto h-8 w-8 text-white flex items-center justify-center" { (icon::x()) }
            }
        }
    }
}