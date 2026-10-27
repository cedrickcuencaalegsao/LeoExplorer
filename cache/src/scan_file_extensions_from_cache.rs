use shared::CacheData;
use shared::FileEntry;
use shared::SupportedFileExtension;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub fn scan_file_extensions_from_cache(cache: &CacheData) -> Vec<SupportedFileExtension> {
    let office_extensions: HashSet<&str> = [
        "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp", "pdf", "rtf", "csv",
    ]
    .iter()
    .copied()
    .collect();

    let dev_extensions: HashSet<&str> = [
        "rs",
        "c",
        "cpp",
        "h",
        "hpp",
        "cs",
        "java",
        "kt",
        "swift",
        "go",
        "py",
        "rb",
        "php",
        "sh",
        "bash",
        "zsh",
        "pl",
        "lua",
        "html",
        "css",
        "js",
        "jsx",
        "ts",
        "tsx",
        "vue",
        "svelte",
        "json",
        "yaml",
        "yml",
        "toml",
        "ini",
        "cfg",
        "env",
        "md",
        "markdown",
        "sql",
        "r",
        "asm",
        "dart",
        "scala",
        "clj",
        "cljs",
        "hs",
        "elm",
        "ex",
        "exs",
        "ml",
        "mli",
        "fs",
        "fsx",
        "fsi",
        "vb",
        "vbs",
        "ps1",
        "psm1",
        "bat",
        "cmd",
        "coffee",
        "less",
        "sass",
        "scss",
        "styl",
        "graphql",
        "gql",
        "proto",
        "thrift",
        "xml",
        "xsd",
        "xslt",
        "dtd",
        "lock",
        "gradle",
        "cmake",
        "make",
        "mk",
        "ninja",
        "dockerfile",
        "gitignore",
        "gitattributes",
        "gitmodules",
        "gitconfig",
        "editorconfig",
        "prettierrc",
        "eslintrc",
        "eslintignore",
        "babelrc",
        "tsconfig",
        "jsconfig",
        "tf",
        "tfvars",
        "tfstate",
        "hcl",
        "jade",
        "pug",
        "ejs",
        "hbs",
        "mustache",
    ]
    .iter()
    .copied()
    .collect();

    let mut extension_counts: HashMap<String, i32> = HashMap::new();

    fn walk_entries(
        entries: &[FileEntry],
        counts: &mut HashMap<String, i32>,
        office: &HashSet<&str>,
        dev: &HashSet<&str>,
    ) {
        for entry in entries {
            if !entry.is_dir {
                if let Some(ext) = Path::new(&entry.name).extension() {
                    if let Some(ext_str) = ext.to_str() {
                        let ext_lower = ext_str.to_lowercase();
                        *counts.entry(ext_lower).or_insert(0) += 1;
                    }
                }
            }
            if let Some(children) = &entry.children {
                walk_entries(children, counts, office, dev);
            }
        }
    }

    walk_entries(
        &cache.roots,
        &mut extension_counts,
        &office_extensions,
        &dev_extensions,
    );

    let mut result: Vec<SupportedFileExtension> = extension_counts
        .into_iter()
        .map(|(extension, count)| {
            let file_type = if office_extensions.contains(extension.as_str()) {
                "office".to_string()
            } else if dev_extensions.contains(extension.as_str()) {
                "dev".to_string()
            } else {
                "other".to_string()
            };
            SupportedFileExtension {
                extension,
                count,
                is_selected: false,
                file_type,
            }
        })
        .collect();

    result.sort_by(|a, b| {
        let rank = |file_type: &str| match file_type {
            "office" => 0,
            "dev" => 1,
            _ => 2,
        };
        let rank_diff = rank(a.file_type.as_str()).cmp(&rank(b.file_type.as_str()));
        if rank_diff != std::cmp::Ordering::Equal {
            return rank_diff;
        }
        a.extension.cmp(&b.extension)
    });

    result
}
