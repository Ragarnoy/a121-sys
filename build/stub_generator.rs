use crate::error::{BuildError, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug)]
struct CFunctionDecl {
    name: String,
    return_type: String,
    parameters: Vec<(String, String)>, // (full declaration, name)
}

pub struct StubGenerator {
    header_files: HashMap<String, Vec<String>>,
    return_values: HashMap<String, String>,
}

impl Default for StubGenerator {
    fn default() -> Self {
        let mut return_values = HashMap::new();
        return_values.insert("bool".to_string(), "true".to_string());
        return_values.insert("uint8_t".to_string(), "0".to_string());
        return_values.insert("uint16_t".to_string(), "0".to_string());
        return_values.insert("uint32_t".to_string(), "0".to_string());
        return_values.insert("float".to_string(), "0.1".to_string());
        return_values.insert("float32_t".to_string(), "0.1".to_string());
        return_values.insert("int32_t".to_string(), "-1".to_string());
        return_values.insert(
            "acc_config_profile_t".to_string(),
            "ACC_CONFIG_PROFILE_3".to_string(),
        );
        return_values.insert(
            "acc_config_idle_state_t".to_string(),
            "ACC_CONFIG_IDLE_STATE_SLEEP".to_string(),
        );
        return_values.insert(
            "acc_config_prf_t".to_string(),
            "ACC_CONFIG_PRF_13_0_MHZ".to_string(),
        );
        return_values.insert(
            "acc_rss_test_state_t".to_string(),
            "ACC_RSS_TEST_STATE_COMPLETE".to_string(),
        );
        return_values.insert(
            "acc_detector_distance_threshold_method_t".to_string(),
            "ACC_DETECTOR_DISTANCE_THRESHOLD_METHOD_FIXED_STRENGTH".to_string(),
        );
        return_values.insert(
            "acc_detector_distance_peak_sorting_t".to_string(),
            "ACC_DETECTOR_DISTANCE_PEAK_SORTING_STRONGEST".to_string(),
        );
        return_values.insert("acc_sensor_id_t".to_string(), "1".to_string());
        return_values.insert(
            "acc_detector_distance_reflector_shape_t".to_string(),
            "ACC_DETECTOR_DISTANCE_REFLECTOR_SHAPE_GENERIC".to_string(),
        );

        let mut header_files = HashMap::new();
        header_files.insert(
            "acconeer_a121_stubs.c".to_string(),
            vec![
                "acc_definitions.h",
                "acc_hal_definitions_a121.h",
                "acc_definitions_common.h",
                "acc_processing.h",
                "acc_sensor.h",
                "acc_config.h",
                "acc_config_subsweep.h",
                "acc_definitions_a121.h",
                "acc_version.h",
                "acc_rss_a121.h",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        if cfg!(feature = "distance") {
            header_files.insert(
                "acc_detector_distance_a121_stubs.c".to_string(),
                vec![
                    "acc_detector_distance_definitions.h",
                    "acc_detector_distance.h",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            );
        }

        if cfg!(feature = "presence") {
            header_files.insert(
                "acc_detector_presence_a121_stubs.c".to_string(),
                vec![
                    "acc_detector_presence.h",
                    "acc_detector_presence_processing.h",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            );
        }

        Self {
            header_files,
            return_values,
        }
    }
}

impl StubGenerator {
    /// Generates one C stub file per entry in `header_files`.
    ///
    /// Returns the names of the functions stubbed in each file, keyed by stub file name.
    pub fn generate_stubs(
        &self,
        include_dir: &Path,
        out_dir: &Path,
    ) -> Result<HashMap<String, Vec<String>>> {
        let mut stubbed = HashMap::new();

        for (stub_file, headers) in &self.header_files {
            let mut functions = Vec::new();

            // Parse all headers for this stub file
            for header in headers {
                let header_path = include_dir.join(header);
                let text = fs::read_to_string(&header_path).map_err(|e| {
                    BuildError::StubGenerationFailed(format!(
                        "Failed to read header {}: {}",
                        header, e
                    ))
                })?;

                functions.extend(self.extract_functions_from_header(&text)?);
            }

            // Generate the stub file
            let stub_content = self.generate_stub_file(&functions, headers)?;

            // Write the stub file
            let stub_path = out_dir.join(stub_file);
            fs::write(&stub_path, stub_content).map_err(|e| {
                BuildError::StubGenerationFailed(format!("Failed to write stub file: {}", e))
            })?;

            stubbed.insert(
                stub_file.clone(),
                functions.into_iter().map(|f| f.name).collect(),
            );
        }
        Ok(stubbed)
    }

    /// Extracts the function prototypes declared at the top level of a C header.
    ///
    /// Comments and preprocessor lines are removed, the remaining text is split into
    /// top-level statements on `;`, and every statement that is not a type definition
    /// and contains a parameter list is parsed as a prototype.
    fn extract_functions_from_header(&self, text: &str) -> Result<Vec<CFunctionDecl>> {
        let code = strip_comments(text);
        let code: String = code
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");

        let mut functions = Vec::new();
        let mut statement = String::new();
        let mut depth = 0usize;

        for c in code.chars() {
            match c {
                '{' => depth += 1,
                '}' => depth = depth.saturating_sub(1),
                ';' if depth == 0 => {
                    let stmt = statement.split_whitespace().collect::<Vec<_>>().join(" ");
                    statement.clear();

                    let is_type = ["typedef", "struct", "enum", "union"]
                        .iter()
                        .any(|kw| stmt.starts_with(kw));
                    if is_type || !stmt.contains('(') {
                        continue;
                    }

                    let func = self.parse_function_declaration(&stmt).ok_or_else(|| {
                        BuildError::StubGenerationFailed(format!(
                            "Failed to parse prototype: {}",
                            stmt
                        ))
                    })?;
                    functions.push(func);
                    continue;
                }
                _ => {}
            }
            statement.push(c);
        }

        Ok(functions)
    }

    fn parse_function_declaration(&self, decl: &str) -> Option<CFunctionDecl> {
        // Split "<return type> <name>(<params>)"
        let (head, rest) = decl.split_once('(')?;
        let params_part = rest.trim().strip_suffix(')')?;

        let (return_type, name) = split_trailing_identifier(head)?;
        if return_type.is_empty() {
            return None;
        }

        // Parse parameters, keeping each declaration verbatim
        let parameters = if params_part.trim().is_empty() || params_part.trim() == "void" {
            Vec::new()
        } else {
            params_part
                .split(',')
                .map(|param| {
                    let param = param.trim();
                    let without_array = param.split('[').next()?;
                    let (_, param_name) = split_trailing_identifier(without_array)?;
                    Some((param.to_string(), param_name))
                })
                .collect::<Option<Vec<_>>>()?
        };

        Some(CFunctionDecl {
            name,
            return_type,
            parameters,
        })
    }

    fn generate_stub_file(
        &self,
        functions: &[CFunctionDecl],
        headers: &[String],
    ) -> Result<String> {
        let mut content = String::new();

        // Add includes
        for header in headers {
            content.push_str(&format!("#include \"{}\"\n", header));
        }

        // Add helper functions
        content.push_str(
            r#"
#include <math.h>
#include <complex.h>
#include <string.h>
#include <stdint.h>

static float fake_external_dependencies(char* foo, complex float iq) {
    char buff[42];
    memcpy(buff, foo, 1);
    memset(foo, 0, 1);
    memmove(buff, foo, 1);
    uint32_t magnitude = (uint32_t) cabsf(iq);
    return roundf(atanf(sinf(cosf(log10f(powf(crealf(iq), 3.14))))));
}
"#,
        );

        // Generate function stubs
        for func in functions {
            content.push_str(&self.generate_function_stub(func));
            content.push_str("\n\n");
        }

        Ok(content)
    }

    fn generate_function_stub(&self, func: &CFunctionDecl) -> String {
        let mut stub = format!("{} {}(", func.return_type, func.name);

        // Add parameters
        if func.parameters.is_empty() {
            stub.push_str("void");
        } else {
            for (i, (param_decl, _)) in func.parameters.iter().enumerate() {
                if i > 0 {
                    stub.push_str(", ");
                }
                stub.push_str(param_decl);
            }
        }
        stub.push_str(") {\n");

        // Add parameter void casts
        for (_, param_name) in &func.parameters {
            stub.push_str(&format!("    (void) {};\n", param_name));
        }

        // Add fake dependencies call for create functions
        if func.name.contains("create") {
            stub.push_str("    char dummy[] = \"dummy\";\n");
            stub.push_str("    fake_external_dependencies(dummy, 1.0 + 2.0*I);\n");
        }

        // Add return value
        if func.return_type != "void" {
            if let Some(return_value) = self.return_values.get(&func.return_type) {
                stub.push_str(&format!("    return {};\n", return_value));
            } else if func.return_type.contains('*') {
                stub.push_str("    return NULL;\n");
            } else {
                stub.push_str(&format!(
                    "    {} result = {{0}};\n    return result;\n",
                    func.return_type
                ));
            }
        }

        stub.push('}');
        stub
    }
}

/// Removes `/* ... */` and `// ...` comments, keeping line breaks.
fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        match (c, chars.peek()) {
            ('/', Some('*')) => {
                chars.next();
                let mut prev = '\0';
                for c in chars.by_ref() {
                    if prev == '*' && c == '/' {
                        break;
                    }
                    if c == '\n' {
                        out.push('\n');
                    }
                    prev = c;
                }
                out.push(' ');
            }
            ('/', Some('/')) => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            }
            _ => out.push(c),
        }
    }

    out
}

/// Splits a declaration such as `const acc_config_t *config` into its type
/// (`const acc_config_t *`) and trailing identifier (`config`).
fn split_trailing_identifier(decl: &str) -> Option<(String, String)> {
    let decl = decl.trim_end();
    let start = decl
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_ascii_alphanumeric() || *c == '_')
        .last()
        .map(|(i, _)| i)?;

    let name = decl[start..].to_string();
    let ty = decl[..start].trim().to_string();
    Some((ty, name))
}
