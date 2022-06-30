use anyhow::anyhow;
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};

pub fn select(message: &str, items: &[String]) -> Result<usize> {
    let mut items = items.to_vec();
    items.sort_by(|a, b| b.cmp(a));
    let selection = match items.len() {
        1 => 0,
        _ => match Select::with_theme(&ColorfulTheme::default())
            .with_prompt(message)
            .default(0)
            .items(items.as_ref())
            .interact()
        {
            Ok(s) => s,
            Err(e) => return Err(anyhow!("{}", e)),
        },
    };

    Ok(selection)
}

pub fn confirm(message: &str) -> Result<()> {
    match Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .interact()
    {
        Ok(is_confirm) => {
            if !is_confirm {
                return Err(anyhow!("not confirmed"));
            }
        }
        Err(e) => {
            log::debug!("confirm interact err: {}", e);
            return Err(anyhow!("confirm interact err: {}", e));
        }
    }
    Ok(())
}
