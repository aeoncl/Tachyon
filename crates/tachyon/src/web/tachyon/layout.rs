use maud::{html, Markup, DOCTYPE};

fn page_head() -> Markup {
    html! {
        head {
            title { "Tachyon" }
            base href="/tachyon/";
            link rel="icon" type="image/x-icon" href="favicon.ico";
            link rel="stylesheet" href="style.css";
            script type="text/javascript" src="jquery-1.10.0.min.js" {}
            script type="text/javascript" src="intercooler-1.2.4.min.js" {}
            script type="text/javascript" src="verify.js" {}
            script type="text/javascript" src="tremove.js" {}
        }
    }
}

fn page_header(show_nav: bool) -> Markup {
    html! {
        div class="header" {
            div class="bg" {
                div class="bg-content" {
                    img class="logo" src="tachyon_logo_2.png" alt="Tachyon Logo";
                    div class="title" {
                        h1 { "Tachyon" }
                        h2 { "Welcome to Tachyon" }
                    }

                    @if show_nav {
                        div class="menu" {
                            ul {
                                li {
                                    a href="/tachyon" { "Home" }
                                }
                            }
                        }
                        div class="signin" {
                            h2 { "Log-on" }
                        }
                    }
                }
            }
        }
    }
}

pub fn layout(content: Markup, show_nav: bool) -> Markup {
    html! {
        (DOCTYPE)
        html {
            (page_head())
            body {
                (page_header(show_nav))
                div class="content" {
                    (content)
                }
            }
        }
    }
}

pub fn tachyon_page(content: Markup) -> Markup {
    layout(content, true)
}

pub fn tachyon_page_no_nav(content: Markup) -> Markup {
    layout(content, false)
}

fn error_page(message: String) -> Markup {
    layout(
        html! {
            h2 { "Something went wrong" }
            p { (message) }
        },
        false,
    )
}