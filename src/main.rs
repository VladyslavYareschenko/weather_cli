use std::io::{stdin, stdout, Write};

use clap::{Parser, Subcommand};
use ini::Ini;
use tokio;

use weather_service_rpc::weather_service_client::*;
use weather_service_rpc::{Location, LocationSearchParams, WeatherQueryParams};

fn select_location(locations: &mut Vec<Location>) -> Location {
    for (index, location) in locations.iter().enumerate() {
        println!("{}. {}, {}, {}", index, location.name, location.state, location.country);
    }

    let mut input = String::new();
    let _ = stdout().flush();
    let read_result = stdin().read_line(&mut input);

    if read_result.is_ok() {
        if let Ok(uindex) = input.trim().parse::<usize>() {
            if uindex < locations.len() {
                return locations.remove(uindex);
            } 
        }
    }
    
    println!("Failed to read location index, try again: ");
    
    return select_location(locations);
}

async fn get_location_for_forecast(
    location_query: String, client_channel: &mut WeatherServiceClient<tonic::transport::Channel>) -> Location {
    let reply = 
        client_channel.get_locations(LocationSearchParams{ query: location_query.clone() }).
            await.expect("Impossible to get locations. ");

    let locations = &mut reply.into_inner().locations;
    
    match locations.len() {
        1 => return locations.remove(0),
        2..=5 => {
            println!("Several locations found, choose one: ");
         
            let loc = select_location(locations);
            println!("Selected: {}, {}, {}", loc.name, loc.state, loc.country);
            return loc;
        }
        0 => {
            panic!("Did not found any location for the '{}' query. \nPlease change the request and try again.", 
                   location_query);
        }
        _ => {
            panic!("Recieved invalid amount of locations!");
        }
    }
}

fn get_current_date_as_string() -> String {
    return chrono::offset::Local::today().format("%m.%d.%Y").to_string();
}

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    GetProviders,
    Configure { provider: String },
    Get { 
        address: String,
        #[clap(default_value_t = String::from(get_current_date_as_string()))]
        date: String
    }
}

static WEATHER_CLI_CONF: &str = ".weather_cli_config";
static WEATHER_CLI_PROVIDER_NAME_KEY: &str = "WEATHER_CLI_PROVIDER_NAME";
static WEATHER_CLI_SERVER_KEY: &str = "WEATHER_CLI_SERVER";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    
    let mut conf = Ini::load_from_file(WEATHER_CLI_CONF).unwrap();

    let address = conf.general_section().get(WEATHER_CLI_SERVER_KEY).unwrap_or("http://[::1]:50051").to_string();
    let mut client = WeatherServiceClient::connect(address).await.expect("Failed to connect to the server.");

    match &args.command {
        Commands::GetProviders => {
            let reply = client.get_weather_providers(()).
                await.expect("An error occurred while receiving the weather providers.").into_inner();

            print!("Available providers: \n{:?}\n", reply.providers);
        },
        Commands::Configure { provider } => {
            let reply = client.get_weather_providers(()).
                await.expect("An error occurred while receiving the weather providers.").into_inner();

            let found = reply.providers.iter().find(|prov| return &provider == prov);
            if found == None {
                panic!("Invalid weather provider passed, list of available providers: \n{:?}", reply.providers);
            }
            
            conf.with_general_section().set(WEATHER_CLI_PROVIDER_NAME_KEY, found.unwrap());
            conf.write_to_file(WEATHER_CLI_CONF).unwrap();

            print!("weather_cli is now ready to do weather forecast with {} provider!\n", provider)
        },
        Commands::Get { address, date } => {            
            let provider_name = conf.general_section().get(WEATHER_CLI_PROVIDER_NAME_KEY).
                expect("Did not found any providers configured. Try to run tool with 'configure' command. ");
            
            let loc = get_location_for_forecast(address.to_owned(), &mut client).await;
            let weather = client.get_weather(WeatherQueryParams {
                    provider: provider_name.to_owned(),
                    location: Some(loc),
                    date: date.to_string()}).
                await.expect("An error occurred while receiving the weather forecast.").into_inner();

            println!("Weather on {}: \nMin temperature: {}, \nMax temperature: {}, \nAvg temperature: {}, \nWeather condition: {}",
                      date, 
                      weather.min_t,
                      weather.max_t,
                      weather.avg_t,
                      weather.condition);
        }
    }

    Ok(())
}
