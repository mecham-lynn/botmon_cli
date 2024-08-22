use std::panic;

use color_eyre::{config::HookBuilder, eyre};

use crate::restore;


// #[derive(Debug, Error)]
// pub enum BotmonError {
    
//     #[error(transparent)]
//     IoErrors(#[from] std::io::Error),
    
//     Report(#[from] e)
// }

pub fn install_hooks() -> color_eyre::Result<()> {
    
    let(panic_hook, eyre_hook) = HookBuilder::default().into_hooks();
    
    //converte from color_eyre hook into a standard hook
    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        restore().unwrap();
        panic_hook(panic_info);
    }));
    
    // convert from a color_eyre Eyrehook to a eyre ErrorHook
    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(
        move |error: &(dyn std::error::Error + 'static)| {
            restore().unwrap();
            eyre_hook(error)
        }
    ))?;
    
    Ok(())
}