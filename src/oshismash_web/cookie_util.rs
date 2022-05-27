use cookie::{Cookie, SameSite, time::Duration};

pub fn create<'a, K, V>(name: K, value: V) -> Cookie<'a>
where
    K: ToString,
    V: ToString
{
    let mut cookie = Cookie::new(name.to_string(), value.to_string());

    cookie.set_secure(true);
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Strict);
    cookie.set_max_age(Duration::seconds(2147483647));

    cookie
}
