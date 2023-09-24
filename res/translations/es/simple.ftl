response-msg =
    { $value ->
        [one] "{ $input }" tiene un paso de Collatz.
       *[other] "{ $input }" tiene { $value } pasos de Collatz.
    }