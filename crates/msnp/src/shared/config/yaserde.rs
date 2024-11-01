use yaserde::ser::Config;

pub const CONFIG_NO_DECL: Config = Config {
    perform_indent: false,
    write_document_declaration: false,
    indent_string: None,
};