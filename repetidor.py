import os

for param_file in os.listdir("parameters_files"):
    stream = os.popen(f'cargo run no_gui_rep "parameters_files/{param_file}" 100000')
