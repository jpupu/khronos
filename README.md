# khronos

With khronos you can rewrite timestamps on log files. Observe:

~~~~
> cat log.txt                                                                                                            
2022-03-20T15:32:01.462 Prepare for celebration
2022-03-20T15:33:00.000 Spring equinox!
2022-03-20T15:33:00.001 Onwards to summer
2022-03-20T16:35:12.414 Starting party phase 2

> cat log.txt | khronos -o unix,.3                                                                                       
1647790321.462 Prepare for celebration
1647790380.000 Spring equinox!
1647790380.001 Onwards to summer
1647794112.414 Starting party phase 2

> khronos log.txt -o delta,ms
0 Prepare for celebration
58538 Spring equinox!
1 Onwards to summer
3732413 Starting party phase 2
~~~~

Reads from stdin lines and rewrites their timestamps. The timestamp must be at the start of line and separated from the message by at least one space. If the timestamp
of a line cannot be successfully parsed, the line is output as-is.

If input format is not given it is automatically deduced from input. In this case the lines are read and output as-is until the first recognizable timestamp is met.

# Usage

~~~~
USAGE:
    khronos [OPTIONS]

OPTIONS:
    -h, --help
            Print help information

    -i, --informat <FMT>
            Input format. Auto-detect if not specified

    -o, --outformat <FMT[,OPTION...]>
            Output format
            
            [default: iso]

INPUT FORMATS:
    iso     ISO 8601
    unix    Unix time in (fractional) seconds
    unixms  Unix time in (fractional) milliseconds

OUTPUT FORMATS:
    iso     ISO 8601. Options: precision, nodate
    unix    Unix time. Options: units, precision
    delta   Time since previous line. Options: units, precision
    elapsed Time since log start. Options: units, precision

OUTPUT OPTIONS:
    precision   .0 | .1 | .2 | ... | .9
    units       s | ms | us | ns
    nodate      nodate

EXAMPLES:
    Specify unix time in milliseconds with 3 fractional digits:
        unix,ms,.3

    Specify delta in seconds with 6 fractional digits:
        delta,.6
~~~~
