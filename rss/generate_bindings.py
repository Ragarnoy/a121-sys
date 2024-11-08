import re
import sys
from pathlib import Path

"""
Script that generate function stubs from Acconeer library header files. 
"""

DEBUG = False
HEADER_PATH = "./include"

FILES = {
    'acconeer_a121_stubs.c': ['acc_hal_definitions_a121.h', 'acc_definitions_common.h', 'acc_processing.h',
                              'acc_sensor.h', 'acc_config.h', 'acc_config_subsweep.h',
                              'acc_definitions_a121.h', 'acc_version.h', 'acc_rss_a121.h'],
    'acc_detector_distance_a121_stubs.c': ['acc_detector_distance_definitions.h', 'acc_detector_distance.h'],
    'acc_detector_presence_a121_stubs.c': ['acc_detector_presence.h'],
}

RETURN_VALUES = {
    'bool': 'true',
    'uint8_t': '0',
    'uint16_t': '0',
    'uint32_t': '0',
    'float': '0.1',
    'int32_t': '-1',
    'acc_config_profile_t': 'ACC_CONFIG_PROFILE_3',
    'acc_config_idle_state_t': 'ACC_CONFIG_IDLE_STATE_SLEEP',
    'acc_config_prf_t': 'ACC_CONFIG_PRF_13_0_MHZ',
    'acc_rss_test_state_t': 'ACC_RSS_TEST_STATE_COMPLETE',
    'acc_detector_distance_threshold_method_t': 'ACC_DETECTOR_DISTANCE_THRESHOLD_METHOD_FIXED_STRENGTH',
    'acc_detector_distance_peak_sorting_t': 'ACC_DETECTOR_DISTANCE_PEAK_SORTING_STRONGEST',
    'acc_sensor_id_t': '1',
    'acc_detector_distance_reflector_shape_t': 'ACC_DETECTOR_DISTANCE_REFLECTOR_SHAPE_GENERIC',
    '': '',
}

EXTRA_CODE = """
#include <math.h>
#include <complex.h>
#include <string.h>
#include <stdint.h>
float fake_external_dependencies(char* foo, complex float iq);
float fake_external_dependencies(char* foo, complex float iq)
{
    char buff[42];
    memcpy(buff, foo, 1);
    memset(foo, 0, 1);
    memmove(buff, foo, 1);
    uint32_t magnitude = (uint32_t) cabsf(iq);
    return roundf(atanf(sinf(cosf(log10f(powf(crealf(iq), 3.14))))));  
}
"""

not_handled_types = set()


def remove_irrelevant_stuff(text):
    # Based on function removeCCppComment from: https://stackoverflow.com/a/18234680

    def blot_out_non_newlines(string):  # Return a string containing only the newline chars contained in strIn
        return "" + ("\n" * string.count('\n'))

    def replacer(match):
        s = match.group(0)
        if s.startswith('/'):  # Matched string is //...EOL or /*...*/  ==> Blot out all non-newline chars
            return blot_out_non_newlines(s)
        else:  # Matched string is '...' or "..."  ==> Keep unchanged
            return s

    pattern1 = re.compile(
        r'//.*?$|/\*.*?\*/|\'(?:\\.|[^\\\'])*\'|"(?:\\.|[^\\"])*"',
        re.DOTALL | re.MULTILINE
    )
    text = re.sub(pattern1, replacer, text)

    pattern2 = re.compile(r'^\s*#.*?$', re.MULTILINE)
    text = re.sub(pattern2, '\n', text)

    pattern3 = re.compile(r'\{[^{]*?}', re.DOTALL | re.MULTILINE)
    while re.search(pattern3, text):
        text = re.sub(pattern3, ' ##CODE_SECTION## ', text)

    return text


def parse_func_parameters(code_string):
    if code_string.strip() == "void":
        return []
    p = re.compile("(.+ \\*?)(\\w+)$")
    params = code_string.split(',')
    res = []
    for param in params:
        m = p.match(param.strip())
        if m:
            res.append({'type': m.group(1), 'name': m.group(2)})
        else:
            print(f"Error: couldn't parse: {params} ")
            sys.exit(1)
    return res


def parse_c_prototype(code_string):
    p_ignore = re.compile("static|typedef")
    if p_ignore.match(code_string):
        if DEBUG:
            print("// Skipping:", code_string)
        return None
    p = re.compile("^(.+ \\*?)(.*)\\((.*)\\)$")
    m = p.search(code_string)
    if m:
        res = {
            'ret_val_type': m.group(1).strip(),
            'name': m.group(2),
            'parsed_params': parse_func_parameters(m.group(3)),
            'param_string': m.group(3),
        }
        return res
    else:
        if DEBUG:
            print("// No match: ", code_string)
        return None


def print_stub(func_info, out_file):
    print(f"{func_info['ret_val_type']} {func_info['name']}({func_info['param_string']})", file=out_file)
    print("{", file=out_file)
    for param in func_info['parsed_params']:
        print(f"  (void) {param['name']};", file=out_file)

    if "create" in func_info['name']:
        print("  fake_external_dependencies(\"dummy\", 1.0 + 2.0*I);", file=out_file)  # Pass dummy values

    if func_info['ret_val_type'] != "void":
        if func_info['ret_val_type'] in RETURN_VALUES:
            print(f"  {func_info['ret_val_type']} res = {RETURN_VALUES[func_info['ret_val_type']]};", file=out_file)
        elif "*" in func_info['ret_val_type']:
            print(f"  {func_info['ret_val_type']}res = NULL;", file=out_file)
        else:
            print(f"  {func_info['ret_val_type']} res;", file=out_file)
            not_handled_types.add(func_info['ret_val_type'])

        print("  return res;", file=out_file)
    print("}", file=out_file)
    print(file=out_file)


def main():
    for out_file_name, header_files in FILES.items():
        print(f"Generating {out_file_name}")
        out_file = open(out_file_name, "w")

        for file in header_files:
            print(f'#include "{file}"', file=out_file)

        print(EXTRA_CODE, file=out_file)

        header_paths = [Path(HEADER_PATH, f) for f in header_files]

        for file in header_paths:
            with open(file, 'r') as f:
                c_code = f.read()

            filtered_c_code = remove_irrelevant_stuff(c_code)

            statements = filtered_c_code.replace('\n', '').replace('\r', '').split(";")

            for statement in statements:
                func_info = parse_c_prototype(statement)
                if func_info:
                    print_stub(func_info, out_file)

        if DEBUG and len(not_handled_types) > 0:
            print(f"//TODO: Fix {not_handled_types=}")


if __name__ == '__main__':
    main()
