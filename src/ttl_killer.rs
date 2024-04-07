use std::time::Duration;

use tokio::{
    fs::{read_dir, remove_file},
    io,
    time::sleep,
};

use crate::{file_data::FileData, state::State};

async fn ttl_killer(state: State) -> Result<(), io::Error> {
    let mut content = read_dir(state.data_directory).await?;
    while let Some(file) = content.next_entry().await? {
        let file_path = file.path();
        let file_data = FileData::read_from(&file_path).await;
        if let Ok(Some(file_data)) = file_data {
            if file_data.expired() {
                remove_file(file_path).await?;
                let upload_path = state.upload_directory.join(file.file_name());
                remove_file(upload_path).await?;
                println!(
                    "INFO: File {} got removed because the ttl was reached.",
                    file.file_name().to_str().unwrap()
                );
            }
        } else if let Err(e) = file_data {
            println!("ERROR: Error while discovering file data: Parsing error while parsing file {}: {e:?}", file.file_name().to_str().unwrap());
        }
    }

    Ok(())
}

pub fn start_ttl_killer(state: State) {
    tokio::spawn(async move {
        println!("INFO: Started ttl killer");
        loop {
            if let Err(e) = ttl_killer(state.clone()).await {
                println!("ERROR: Error while running ttl killer: {e:?}");
            }
            sleep(Duration::from_secs(10)).await;
        }
    });
}
