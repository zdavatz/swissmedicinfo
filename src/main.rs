use quick_xml::events::Event;
use quick_xml::reader::Reader;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::env;
use std::fs::File;
use std::io::{BufReader, Write, Cursor};
use std::path::Path;
use zip::ZipArchive;

const DOWNLOAD_URL: &str = "https://download.swissmedicinfo.ch/";

#[derive(Debug)]
struct Record {
    identifier: String,
    date: String,
}

fn download_latest_xml() -> Result<String, Box<dyn std::error::Error>> {
    println!("Downloading latest XML from SwissMedicInfo...");
    
    let client = Client::builder()
        .cookie_store(true)
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:65.0) Gecko/20100101 Firefox/65.0")
        .build()?;
    
    // First request to get cookies and ViewState values
    println!("Fetching download page...");
    let initial_response = client
        .get(DOWNLOAD_URL)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
        .header("Accept-Language", "de,en-US;q=0.7,en;q=0.3")
        .send()?;
    
    let html_content = initial_response.text()?;
    let document = Html::parse_document(&html_content);
    
    // Extract ViewState values
    let viewstate_selector = Selector::parse(r#"input[name="__VIEWSTATE"]"#).unwrap();
    let viewstate_gen_selector = Selector::parse(r#"input[name="__VIEWSTATEGENERATOR"]"#).unwrap();
    let event_validation_selector = Selector::parse(r#"input[name="__EVENTVALIDATION"]"#).unwrap();
    
    let viewstate = document
        .select(&viewstate_selector)
        .next()
        .and_then(|el| el.value().attr("value"))
        .ok_or("Could not find __VIEWSTATE")?;
    
    let viewstate_gen = document
        .select(&viewstate_gen_selector)
        .next()
        .and_then(|el| el.value().attr("value"))
        .ok_or("Could not find __VIEWSTATEGENERATOR")?;
    
    let event_validation = document
        .select(&event_validation_selector)
        .next()
        .and_then(|el| el.value().attr("value"))
        .ok_or("Could not find __EVENTVALIDATION")?;
    
    println!("Submitting download request...");
    
    // Prepare POST data
    let post_data = format!(
        "__VIEWSTATE={}&__VIEWSTATEGENERATOR={}&__EVENTVALIDATION={}&ctl00%24MainContent%24BtnYes=Ja",
        urlencoding::encode(viewstate),
        urlencoding::encode(viewstate_gen),
        urlencoding::encode(event_validation)
    );
    
    // Submit the form
    let download_response = client
        .post(DOWNLOAD_URL)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
        .header("Accept-Language", "de,en-US;q=0.7,en;q=0.3")
        .header("Referer", DOWNLOAD_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Upgrade-Insecure-Requests", "1")
        .body(post_data)
        .send()?;
    
    // Get the ZIP file content
    let zip_bytes = download_response.bytes()?;
    println!("Downloaded {} bytes", zip_bytes.len());
    
    // Save the ZIP file first
    let today = chrono::Local::now();
    let zip_filename = format!("AipsDownload_{}.zip", today.format("%Y%m%d"));
    let mut zip_file = File::create(&zip_filename)?;
    zip_file.write_all(&zip_bytes)?;
    println!("Saved ZIP to {}", zip_filename);
    
    // Extract the ZIP file
    println!("Extracting ZIP file...");
    let cursor = Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor)?;
    
    let mut xml_filename = String::new();
    
    // Extract all files and find the XML file
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = file.name().to_string();  // Convert to owned String immediately
        
        println!("Extracting: {}", outpath);
        
        if outpath.ends_with(".xml") && outpath.contains("AipsDownload") {
            xml_filename = format!("AipsDownload_{}.xml", today.format("%Y%m%d"));
            let mut outfile = File::create(&xml_filename)?;
            std::io::copy(&mut file, &mut outfile)?;
            println!("Extracted XML to {}", xml_filename);
        } else if outpath.ends_with(".xsd") {
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
            println!("Extracted XSD to {}", outpath);
        }
    }
    
    if xml_filename.is_empty() {
        return Err("No XML file found in ZIP archive".into());
    }
    
    println!("Download and extraction complete!");
    
    Ok(xml_filename)
}

fn parse_xml(file_path: &str) -> Result<Vec<Record>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let buf_reader = BufReader::new(file);
    let mut reader = Reader::from_reader(buf_reader);
    reader.config_mut().trim_text(true);
    
    let mut records = Vec::new();
    let mut buf = Vec::new();
    
    let mut in_bundle = false;
    let mut in_date = false;
    let mut in_regulated_auth = false;
    let mut in_identifier = false;
    
    let mut current_date = String::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                let local_name = name.as_ref();
                
                match local_name {
                    b"MedicinalDocumentsBundle" => {
                        in_bundle = true;
                        current_date.clear();
                    }
                    b"Date" if in_bundle => {
                        in_date = true;
                    }
                    b"RegulatedAuthorization" if in_bundle => {
                        in_regulated_auth = true;
                    }
                    b"Identifier" if in_regulated_auth => {
                        in_identifier = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape()?.into_owned();
                
                if in_date {
                    // Extract date part from datetime (format: YYYY-MM-DD)
                    current_date = text.split('T').next().unwrap_or("").to_string();
                }
                
                if in_identifier {
                    let identifier = text.trim().to_string();
                    // Only include 5-digit identifiers
                    if identifier.len() == 5 && identifier.chars().all(|c| c.is_ascii_digit()) {
                        records.push(Record {
                            identifier,
                            date: current_date.clone(),
                        });
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                let local_name = name.as_ref();
                
                match local_name {
                    b"MedicinalDocumentsBundle" => {
                        in_bundle = false;
                    }
                    b"Date" => {
                        in_date = false;
                    }
                    b"RegulatedAuthorization" => {
                        in_regulated_auth = false;
                    }
                    b"Identifier" => {
                        in_identifier = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Box::new(e)),
            _ => {}
        }
        buf.clear();
    }
    
    Ok(records)
}

fn write_csv(records: &[Record], output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_path(output_file)?;
    
    // Write header
    wtr.write_record(&["identifier", "date"])?;
    
    // Write records
    for record in records {
        wtr.write_record(&[&record.identifier, &record.date])?;
    }
    
    wtr.flush()?;
    Ok(())
}

fn extract_date_from_filename(filename: &str) -> String {
    // Expected format: AipsDownload_YYYYMMDD.xml
    if let Some(date_part) = filename.strip_prefix("AipsDownload_") {
        if let Some(date_str) = date_part.strip_suffix(".xml") {
            if date_str.len() == 8 {
                let year = &date_str[0..4];
                let month = &date_str[4..6];
                let day = &date_str[6..8];
                return format!("{}.{}.{}", day, month, year);
            }
        }
    }
    
    // Fallback to current date
    chrono::Local::now().format("%d.%m.%Y").to_string()
}

fn parse_date_filter(date_str: &str) -> Result<String, String> {
    // Expected format: DD.MM.YYYY
    let parts: Vec<&str> = date_str.split('.').collect();
    if parts.len() != 3 {
        return Err("Date must be in format DD.MM.YYYY".to_string());
    }
    
    let day = parts[0];
    let month = parts[1];
    let year = parts[2];
    
    if day.len() != 2 || month.len() != 2 || year.len() != 4 {
        return Err("Date must be in format DD.MM.YYYY".to_string());
    }
    
    // Convert to YYYY-MM-DD for comparison
    Ok(format!("{}-{}-{}", year, month, day))
}

fn parse_threshold_filter(threshold_str: &str) -> Result<u32, String> {
    threshold_str.parse::<u32>()
        .map_err(|_| "Threshold must be a valid number".to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <xml_file|--download> [--since DD.MM.YYYY] [--larger THRESHOLD] [--today]", args[0]);
        eprintln!("Examples:");
        eprintln!("  {} AipsDownload_20260130.xml", args[0]);
        eprintln!("  {} AipsDownload_20260130.xml --since 01.01.2025", args[0]);
        eprintln!("  {} --download", args[0]);
        eprintln!("  {} --download --since 01.01.2025", args[0]);
        eprintln!("  {} --download --larger 5000", args[0]);
        eprintln!("  {} --download --since 01.01.2025 --larger 5000", args[0]);
        eprintln!("  {} --download --today", args[0]);
        std::process::exit(1);
    }
    
    // Check if we need to download
    let xml_file = if args[1] == "--download" {
        download_latest_xml()?
    } else {
        args[1].clone()
    };
    
    if !Path::new(&xml_file).exists() {
        eprintln!("Error: File '{}' not found", xml_file);
        std::process::exit(1);
    }
    
    // Check for --since option (can be at position 2 or 3 depending on --download)
    let since_date = if args.len() >= 3 {
        let since_pos = args.iter().position(|arg| arg == "--since");
        if let Some(pos) = since_pos {
            if args.len() > pos + 1 {
                match parse_date_filter(&args[pos + 1]) {
                    Ok(date) => Some((date, args[pos + 1].clone())),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("Error: --since requires a date argument");
                std::process::exit(1);
            }
        } else {
            None
        }
    } else {
        None
    };
    
    // Check for --larger option
    let larger_threshold = if args.len() >= 3 {
        let larger_pos = args.iter().position(|arg| arg == "--larger");
        if let Some(pos) = larger_pos {
            if args.len() > pos + 1 {
                match parse_threshold_filter(&args[pos + 1]) {
                    Ok(threshold) => Some(threshold),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("Error: --larger requires a threshold number");
                std::process::exit(1);
            }
        } else {
            None
        }
    } else {
        None
    };
    
    // Check for --today option
    let today_mode = args.iter().any(|arg| arg == "--today");
    
    // Extract filename from path
    let filename = Path::new(&xml_file)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("AipsDownload_20260130.xml");
    
    let output_date = extract_date_from_filename(filename);
    let output_file = if today_mode {
        "today".to_string()
    } else if let Some(threshold) = larger_threshold {
        format!("larger_{}_{}.csv", threshold, output_date)
    } else {
        format!("swissmedicinfo_{}.csv", output_date)
    };
    
    println!("Parsing {}...", xml_file);
    let mut records = parse_xml(&xml_file)?;
    
    println!("Found {} records with 5-digit identifiers", records.len());
    
    // Count unique identifiers
    let mut unique_identifiers = std::collections::HashSet::new();
    for record in &records {
        unique_identifiers.insert(&record.identifier);
    }
    println!("Unique identifiers: {}", unique_identifiers.len());
    
    // Filter by date if --since was specified
    if let Some((ref filter_date, ref display_date)) = since_date {
        let original_count = records.len();
        records.retain(|r| r.date >= *filter_date);
        println!("Filtered to {} records since {}", records.len(), display_date);
        println!("(excluded {} older records)", original_count - records.len());
    }
    
    // If --today is specified, filter to today's date only and get unique identifiers
    if today_mode {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let original_count = records.len();
        records.retain(|r| r.date == today);
        println!("Filtered to {} records from today ({})", records.len(), today);
        println!("(excluded {} records from other dates)", original_count - records.len());
        
        // Get unique identifiers and normalize to 5 digits
        let mut unique_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        for record in &records {
            // Parse identifier and ensure it's 5 digits
            if let Ok(id_num) = record.identifier.parse::<u32>() {
                // Format as 5 digits with leading zeros
                let normalized = format!("{:05}", id_num);
                unique_ids.insert(normalized);
            }
        }
        
        // Convert to sorted vector
        let mut sorted_ids: Vec<String> = unique_ids.into_iter().collect();
        sorted_ids.sort();
        
        println!("Found {} unique identifiers for today", sorted_ids.len());
        println!("Writing to {}...", output_file);
        
        // Write output file - just numbers, one per line, no header
        let mut file = File::create(&output_file)?;
        for id in sorted_ids {
            writeln!(file, "{}", id)?;
        }
        
        println!("Done! Output written to {}", output_file);
        
        // Copy file to remote server
        println!("Copying to remote server zdavatz@65.109.137.20:/var/www/pillbox.oddb.org/");
        let scp_status = std::process::Command::new("scp")
            .arg(&output_file)
            .arg("zdavatz@65.109.137.20:/var/www/pillbox.oddb.org/")
            .status();
        
        match scp_status {
            Ok(status) if status.success() => {
                println!("Successfully copied to remote server!");
            }
            Ok(status) => {
                eprintln!("Warning: scp command failed with status: {}", status);
            }
            Err(e) => {
                eprintln!("Warning: Failed to execute scp: {}", e);
            }
        }
        
        return Ok(());
    }
    
    // If --larger is specified, filter to unique identifiers above threshold
    if let Some(threshold) = larger_threshold {
        // Get unique identifiers that are larger than threshold
        let mut unique_filtered: std::collections::HashSet<String> = std::collections::HashSet::new();
        for record in &records {
            if let Ok(id_num) = record.identifier.parse::<u32>() {
                if id_num > threshold {
                    unique_filtered.insert(record.identifier.clone());
                }
            }
        }
        
        println!("Found {} unique identifiers larger than {}", unique_filtered.len(), threshold);
        
        // Keep only the most recent record for each unique identifier
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        records.retain(|r| {
            if unique_filtered.contains(&r.identifier) && !seen.contains(&r.identifier) {
                seen.insert(r.identifier.clone());
                true
            } else {
                false
            }
        });
    }
    
    // Sort by date descending (most recent first)
    records.sort_by(|a, b| b.date.cmp(&a.date));
    
    println!("Writing to {}...", output_file);
    
    write_csv(&records, &output_file)?;
    println!("Done! Output written to {}", output_file);
    
    Ok(())
}
