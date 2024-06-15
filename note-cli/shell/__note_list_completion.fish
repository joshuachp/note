function __note_list_completion -d "Complete with the list of notes in a directory"
    set -l token (commandline -ct)

    if test $token != ''
        $CARGO_TARGET_DIR/debug/note list $token
    else
        $CARGO_TARGET_DIR/debug/note list
    end
end
