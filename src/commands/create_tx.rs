use crate::templates;
use anyhow::{bail, Result};
use std::fs;
use std::ops::Deref;
use std::path::Path;

pub fn parse_fields(args: &[String]) -> Vec<(String, String)> {
    args.chunks(2)
        .map(|chunk| {
            if chunk.len() == 2 {
                (chunk[0].clone(), chunk[1].clone())
            } else {
                (chunk[0].clone(), "String".to_string())
            }
        })
        .collect()
}

pub fn create_transaction(
    project_path: &str,
    tx_name: &str,
    fields: Vec<(String, String)>,
) -> Result<()> {
    let path = Path::new(project_path);
    if !path.exists() {
        bail!("Project directory not found. Make sure you're in the correct directory.");
    }

    let tx_content = modify_tx_file(tx_name, &fields)?;
    fs::write(path.join("src").join("tx.rs"), tx_content)?;

    let state_content = modify_state_file(tx_name, &fields)?;
    fs::write(path.join("src").join("state.rs"), state_content)?;

    print_transaction_info(tx_name, &fields);
    Ok(())
}

fn print_transaction_info(tx_name: &str, fields: &[(String, String)]) {
    println!("✨ Created new transaction type: {}", tx_name);
    println!("Transaction fields:");
    for (name, type_) in fields {
        println!("  {}: {}", name, type_);
    }
    println!("\nUpdate the verify() and process() methods in src/tx.rs to add your custom logic!");
}

fn modify_tx_file(tx_name: &str, fields: &[(String, String)]) -> Result<String> {
    let tx_file = templates::TX_RS;

    // new enum variat
    let fields_struct = fields
        .iter()
        .map(|(name, type_)| format!("        {}: {}", name, type_))
        .collect::<Vec<_>>()
        .join(",\n");

    let new_variant = if fields.is_empty() {
        format!("    {},\n    Noop", tx_name)
    } else {
        format!("    {} {{\n{}\n    }},\n    Noop", tx_name, fields_struct)
    };

    let modified = tx_file.replace(
        "#[derive(Subcommand, Clone, Serialize, Deserialize, Debug)]\npub enum Transaction {\n    Noop,\n}",
        &format!(
            "#[derive(Subcommand, Clone, Serialize, Deserialize, Debug)]\npub enum Transaction {{\n{}\n}}",
            new_variant
        ),
    );

    let verify_match = format!(
        r#"        match self {{
            Self::{} {{ {} }} => {{
                // TODO: Add verification logic here
                Ok(())
            }},
            Self::Noop => Ok(()),
        }}"#,
        tx_name,
        fields
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let modified = modified.replace(
        "    pub fn verify(&self) -> Result<()> {\n        match self {\n            Transaction::Noop => Ok(()),\n        }\n    }",
        &format!("    pub fn verify(&self) -> Result<()> {{\n{}\n    }}", verify_match)
    );

    Ok(modified)
}

fn modify_state_file(tx_name: &str, fields: &[(String, String)]) -> Result<String> {
    let state_file = templates::STATE_RS;

    let fields_pattern = fields
        .iter()
        .map(|(name, _)| name.deref())
        .collect::<Vec<_>>()
        .join(", ");

    let validate_match = format!(
        r#"        match tx {{
            Transaction::{} {{ {} }} => Ok(()),
            Transaction::Noop => Ok(()),
        }}"#,
        tx_name, fields_pattern
    );

    let process_match = format!(
        r#"        match tx {{
            Transaction::{} {{ {} }} => Ok(()),
            Transaction::Noop => Ok(()),
        }}"#,
        tx_name, fields_pattern
    );

    // Replace validate_tx match
    let modified = state_file.replace(
        "        match tx {\n            Transaction::Noop => Ok(()),\n        }",
        &validate_match,
    );

    // Replace process_tx match
    let modified = modified.replace(
        "        match tx {\n            Transaction::Noop => Ok(()),\n        }",
        &process_match,
    );

    Ok(modified)
}