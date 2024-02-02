use std::net::IpAddr;
mod fronious;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ip = IpAddr::V4(std::net::Ipv4Addr::new(10, 69, 0, 50));
    println!("{:#?}", fronious::get_api_version(ip)?);
    //println!("{:#?}", fronious::get_inverter_realtime_data(ip, fronious::Scope::System)?);
    Ok(())
}
