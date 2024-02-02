use std::net::IpAddr;

use fronius::{DeviceId, Fronius, ThreePhaseInverterData};
mod fronius;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ip = IpAddr::V4(std::net::Ipv4Addr::new(10, 69, 0, 50));
    let fronius = Fronius::connect(ip)?;
    println!("{:#?}", fronius.get_power_flow_realtime_data()?);
    //println!("{:#?}", fronious::get_inverter_realtime_data(ip, fronious::Scope::System)?);
    Ok(())
}
