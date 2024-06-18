function __note_list_completion -d "Complete with the list of notes in a directory"
    set -l token (commandline -ct)

    if test $token != ''
        note list $token
    else
        note list
    end
end
