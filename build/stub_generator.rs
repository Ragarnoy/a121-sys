use crate::error::{BuildError, Result};
use bindgen::Builder;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug)]
struct CFunctionDecl {
    name: String,
    return_type: String,
    parameters: Vec<(String, String)>, // (type, name)
}

#[derive(Debug, Default)]
struct FunctionCollector {
    _functions: Vec<CFunctionDecl>,
}

impl bindgen::callbacks::ParseCallbacks for FunctionCollector {
    fn item_name(&self, _name: &str) -> Option<String> {
        None
    }

    fn header_file(&self, _filename: &str) {
        // Implement if needed for debugging
    }
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
                vec!["acc_detector_presence.h"]
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
    pub fn generate_stubs(&self, include_dir: &Path, out_dir: &Path) -> Result<()> {
        for (stub_file, headers) in &self.header_files {
            let mut functions = Vec::new();

            // Parse all headers for this stub file
            for header in headers {
                let header_path = include_dir.join(header);
                fs::read_to_string(&header_path).map_err(|e| {
                    BuildError::StubGenerationFailed(format!(
                        "Failed to read header {}: {}",
                        header, e
                    ))
                })?;

                let collector = FunctionCollector::default();

                let bindings = Builder::default()
                    .header(header_path.to_str().unwrap())
                    .parse_callbacks(Box::new(collector))
                    .clang_arg("-I/usr/lib/arm-none-eabi/include")
                    .clang_arg(format!("-I{}", include_dir.display()))
                    .generate()
                    .map_err(|e| {
                        BuildError::StubGenerationFailed(format!(
                            "Failed to generate bindings: {}",
                            e
                        ))
                    })?;

                // Extract function declarations by parsing the generated bindings text
                let bindings_text = bindings.to_string();
                functions.extend(self.extract_functions_from_text(&bindings_text)?);
            }

            // Generate the stub file
            let stub_content = self.generate_stub_file(&functions, headers)?;

            // Write the stub file
            let stub_path = out_dir.join(stub_file);
            fs::write(&stub_path, stub_content).map_err(|e| {
                BuildError::StubGenerationFailed(format!("Failed to write stub file: {}", e))
            })?;
        }
        Ok(())
    }

    fn extract_functions_from_text(&self, text: &str) -> Result<Vec<CFunctionDecl>> {
        let mut functions = Vec::new();

        // Split the text into lines and look for extern "C" function declarations
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with("extern") && line.contains("fn") {
                if let Some(func) = self.parse_function_declaration(line) {
                    functions.push(func);
                }
            }
        }

        Ok(functions)
    }

    fn parse_function_declaration(&self, line: &str) -> Option<CFunctionDecl> {
        // Basic function declaration parsing
        let line = line.trim_start_matches("extern \"C\" ").trim();
        if !line.starts_with("fn ") {
            return None;
        }

        let line = line.trim_start_matches("fn ").trim_end_matches(';');

        // Split name and parameters
        let mut parts = line.splitn(2, '(');
        let name = parts.next()?.trim().to_string();
        let params_part = parts.next()?.trim_end_matches(')');

        // Parse parameters
        let parameters = if params_part.trim() == "void" {
            Vec::new()
        } else {
            params_part
                .split(',')
                .filter_map(|param| {
                    let mut parts = param.trim().rsplitn(2, ' ');
                    let param_name = parts.next()?.to_string();
                    let param_type = parts.next()?.to_string();
                    Some((param_type, param_name))
                })
                .collect()
        };

        // Parse return type
        let return_type = if line.contains("->") {
            line.split("->").nth(1)?.trim().to_string()
        } else {
            "void".to_string()
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

float fake_external_dependencies(char* foo, complex float iq) {
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
            for (i, (param_type, param_name)) in func.parameters.iter().enumerate() {
                if i > 0 {
                    stub.push_str(", ");
                }
                stub.push_str(&format!("{} {}", param_type, param_name));
            }
        }
        stub.push_str(") {\n");

        // Add parameter void casts
        for (_, param_name) in &func.parameters {
            stub.push_str(&format!("    (void) {};\n", param_name));
        }

        // Add fake dependencies call for create functions
        if func.name.contains("create") {
            stub.push_str("    fake_external_dependencies(\"dummy\", 1.0 + 2.0*I);\n");
        }

        // Add return value
        if func.return_type != "void" {
            if let Some(return_value) = self.return_values.get(&func.return_type) {
                stub.push_str(&format!("    return {};\n", return_value));
            } else if func.return_type.contains('*') {
                stub.push_str("    return NULL;\n");
            } else {
                stub.push_str(&format!(
                    "    {} result = {{}};\n    return result;\n",
                    func.return_type
                ));
            }
        }

        stub.push('}');
        stub
    }
}
