#[macro_use]
extern crate lazy_static;

use clap::Parser;

use crate::error::{Result};


mod contura;
mod dovre;
mod nordpeis;
mod morsoe;
mod westbo;
mod termatech;
mod jotul;
mod scanspis;
mod keddy;
mod rais;
mod shiedel;
mod sitemap;
mod error;
mod download;

#[derive(Parser, Debug, Default)]
struct CLArgs {
    #[arg(long, default_value_t = false)]
    contura   : bool,
    #[arg(long, default_value_t = false)]
    dovre   : bool,
    #[arg(long, default_value_t = false)]
    jotul     : bool,
    #[arg(long, default_value_t = false)]
    keddy     : bool,
    #[arg(long, default_value_t = false)]
    morsoe    : bool,
    #[arg(long, default_value_t = false)]
    nordpeis  : bool,
    #[arg(long, default_value_t = false)]
    rais  : bool,
    #[arg(long, default_value_t = false)]
    scanspis  : bool,
    #[arg(long, default_value_t = false)]
    shiedel  : bool,
    #[arg(long, default_value_t = false)]
    termatech : bool,
    #[arg(long, default_value_t = false)]
    westbo    : bool,
}

fn main() -> Result<()>{
    
    let mut args = CLArgs::parse();

    if !args.contura &&
    !args.dovre &&
    !args.jotul &&
    !args.keddy &&
    !args.morsoe &&
    !args.nordpeis &&
    !args.rais &&
    !args.scanspis &&
    !args.shiedel &&
    !args.termatech &&
    !args.westbo {
        args.contura = true;
        args.dovre = true;
        args.jotul = true;
        args.keddy = true;
        args.morsoe = true;
        args.nordpeis = true;
        args.rais = true;
        args.scanspis = true;
        args.shiedel = true;
        args.termatech = true;
        args.westbo = true;
    }

    let mut tasks : Vec<fn() -> ()> = vec![];

    if args.contura {
        tasks.push(contura::run);
    }
    if args.dovre {
        tasks.push(dovre::run);
    }
    if args.jotul {
        tasks.push(jotul::run);
    }
    if args.nordpeis {
        tasks.push(nordpeis::run);
    }
    if args.keddy {
        tasks.push(keddy::run);
    }
    if args.morsoe {
        tasks.push(morsoe::run);
    }
    if args.rais {
        tasks.push(rais::run);
    }
    if args.scanspis {
        tasks.push(scanspis::run);
    }
    if args.shiedel {
        tasks.push(shiedel::run);
    }
    if args.termatech {
        tasks.push(termatech::run);
    }
    if args.westbo {
        tasks.push(westbo::run);
    }

    for t in tasks {
        t();
    }

    Ok(())
}
