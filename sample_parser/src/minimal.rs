use std::{env, fs::File, io::Read, sync::Arc};

use llguidance::{api::TopLevelGrammar, toktrie::ApproximateTokEnv, Matcher, ParserFactory};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <schema.ll.json> <sample.json>", args[0]);
        std::process::exit(1);
    }

    let schema_file = read_file_to_string(&args[1]);
    let schema: TopLevelGrammar = if args[1].ends_with(".ll.json") {
        serde_json::from_str(&schema_file).expect("Invalid JSON in schema")
    } else if args[1].ends_with(".schema.json") {
        let val = serde_json::from_str(&schema_file).expect("Invalid JSON in schema");
        TopLevelGrammar::from_json_schema(val)
    } else {
        panic!("Unknown schema file extension")
    };
    let obj_str = read_file_to_string(&args[2]);

    // typically you would use toktrie_hf_tokenizers or implement this yourself
    let tok_env = ApproximateTokEnv::single_byte_env();

    let mut factory = ParserFactory::new_simple(&tok_env).unwrap();

    // set to 2 for more output; 1 is warnings only
    factory.set_stderr_log_level(1);

    // after initial setup, the factory can be read-only
    let factory = Arc::new(factory);

    let tokens = tok_env.tokenize(&obj_str);

    let parser = factory.create_parser(schema);
    let mut constraint = Matcher::new(parser);

    let trie = tok_env.tok_trie();

    eprintln!("Parsing tokens: {}", trie.tokens_dbg(&tokens));

    let mut idx = 0;
    while idx < tokens.len() {
        let mask = constraint.compute_mask().unwrap();

        let sampled_token = tokens[idx];
        assert!(mask.is_allowed(sampled_token));

        constraint.consume_token(sampled_token).unwrap();
        idx += 1;

        let splice = constraint.compute_ff_tokens();

        // The splice contains the tokens that the parser wants to append to the output.

        // if this fails, our test data is broken
        if tokens[idx..idx + splice.len()] != splice {
            panic!(
                "BAD TEST: ff_tokens mismatch:\n{}\n{}",
                trie.tokens_dbg(&tokens[idx..idx + splice.len()]),
                trie.tokens_dbg(&splice)
            );
        }

        if splice.len() > 1 {
            println!("FF: {}", trie.tokens_dbg(&splice));
            constraint.consume_tokens(&splice).unwrap();
            idx += splice.len();
        }
    }

    // the stop reason should be likely also sent to the user
    println!("Stop reason: {:?}", constraint.stop_reason());
}

fn read_file_to_string(filename: &str) -> String {
    let mut file = File::open(filename).expect("Unable to open file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Unable to read file");
    content
}
