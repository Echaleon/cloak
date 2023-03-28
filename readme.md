# Cloak

A simple tool written in rust to hide files, folders and symlinks, by marking them as hidden on Windows, or by prepending a dot to the filename on Unix systems.

By default, the tool will hide all files and folders in the current directory, but you can specify a path to hide files in a different directory. You can specify glob and regex patterns to include or exclude to filter. Additionally, recursive searching can be enabled.

There is an additional watch mode that will watch the specified directory for changes and hide files as they are created or renamed.

## Usage

```
Usage: cloak.exe [OPTIONS] [PATH]...

Arguments:
  [PATH]...  Path(s) to the directory to hide files and folders in. Defaults to the current directory. (default: ".")

Options:
  -r, --recursive                      Flag for recursive search and watch (default: false)
  -w, --watch                          Flag to watch for changes, rather than just run once (default: false)
  -m, --test                           Flag to enable test mode, which will not actually hide files or folders. (default: false)
  -v, --verbose                        Flag to enable verbose mode, which will print out more information. (default: false)     
  -p, --pattern <PATTERN>              Glob pattern to match files and folders to hide. Can be specified multiple times to add  
                                       more patterns. These are matched after glob and regex exclude patterns, but before regex 
                                       patterns. By default, all files and folders are hidden. (default: ["*"])
  -x, --exclude <EXCLUDE>              Glob pattern to exclude files and folders from hiding. Can be specified multiple times to
                                       add more patterns. These are matched first, before regex exclude patterns, and glob and  
                                       regex patterns. By default, no files or folders are excluded. (default: [])
  -g, --regex <REGEX>                  Regex pattern to match files and folders to hide. Can be specified multiple times to add 
                                       more patterns. Regex patterns are matched against the full path of the file or folder.   
                                       They are matched last, after glob and regex exclude patterns, and glob patterns. By       
                                       default, all files and folders are hidden. (default: [".*"])
  -e, --regex-exclude <REGEX_EXCLUDE>  Regex pattern to exclude files and folders from hiding. Can be specified multiple times to
                                       add more patterns. Regex patterns are matched against the full path of the file or folder.
                                       They are matched after glob exclude patterns, but before glob and regex patterns. By      
                                       default, no files or folders are excluded. (default: [])
  -t, --types <TYPES>                  Types of objects to hide. Can be specified multiple times to add more types. By default,  
                                       all types are hidden. (default: ["file", "folder", "symlink"]) [possible values: file,    
                                       folder, symlink]
  -j, --threads <THREADS>              Set the number of threads to use in the thread pool. Still will spawn a small number of   
                                       threads for other tasks. (default: number of logical cores)
  -h, --help                           Print help
  -V, --version                        Print version
```
