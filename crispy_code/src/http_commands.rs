use crate::pattern::NamedPattern;
use reqwest;
use reqwest::header::CONTENT_TYPE;

pub static HTTP_LISTEN_PORT: u16 = 3000;

pub fn start(pattern: NamedPattern) -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    client
        .post(format!(
            "http://127.0.0.1:{}/start/{}",
            HTTP_LISTEN_PORT, pattern.name
        ))
        .header(CONTENT_TYPE, "application/json")
        .json(&pattern)
        .send()?;
    Ok(())
}

pub fn stop(pattern: NamedPattern) -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    client
        .post(format!(
            "http://127.0.0.1:{}/stop/{}",
            HTTP_LISTEN_PORT, pattern.name
        ))
        .header(CONTENT_TYPE, "application/json")
        .json(&pattern)
        .send()?;
    Ok(())
}

pub fn stopall() -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    client
        .post(format!("http://127.0.0.1:{}/stopall", HTTP_LISTEN_PORT))
        .header(CONTENT_TYPE, "application/json")
        .send()?;
    Ok(())
}

pub fn clear(pattern: NamedPattern) -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    client
        .post(format!(
            "http://127.0.0.1:3000/{}/{}",
            HTTP_LISTEN_PORT, pattern.name
        ))
        .header(CONTENT_TYPE, "application/json")
        .json(&pattern)
        .send()?;
    Ok(())
}

pub fn clearall() -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    client
        .post(format!("http://127.0.0.1:{}/clearall", HTTP_LISTEN_PORT))
        .header(CONTENT_TYPE, "application/json")
        .send()?;
    Ok(())
}
