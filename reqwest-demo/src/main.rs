use std::fs::File;
use std::io::copy;
use reqwest::blocking::get;

fn download_file() -> Result<(), Box<dyn std::error::Error>> {
    // URL of the file
    let url = "http://localhost:3000/static/db.zip";

    // Send GET request
    let response = get(url)?;

    // Open a file to write the downloaded content
    let mut file = File::create("db.zip")?;

    // Copy the content from the response to the file
    let content = response.bytes()?;
    copy(&mut content.as_ref(), &mut file)?;

    println!("File downloaded successfully!");

    Ok(())
}

fn main() {
    if let Err(e) = download_file() {
        eprintln!("Error: {}", e);
    }
}
