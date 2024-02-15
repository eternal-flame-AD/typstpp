#align(center)[
  *Typstpp Demo*
]

Load some libraries:

```r
#| message: false
library(tidyverse)
```


Then make a plot:

```r
plot(iris)
```

Then try some Haskell:

```hs
:{
  fib :: Int -> Int
  fib 0 = 0
  fib 1 = 1
  fib n = fib (n-1) + fib (n-2)
:}

map fib [0..10]
```

Then make a table:

```r
knitr::kable(head(iris))
```