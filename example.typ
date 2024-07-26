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

Mix some code, plots and tables in the same chunk:

```r
factorial <- function(n) {
  if (n == 0) {
    return(1)
  } else {
    return(n * factorial(n - 1))
  }
}

x <- 1:10
y <- sapply(x, factorial)

plot(x, y, type = "l")

print("↑ base R plot ↓ ggplot2 plot")

ggplot(data.frame(x = x, y = y), aes(x, y)) +
  geom_line() +
  labs(title = "Factorial function", x = "x", y = "y")

knitr::kable(data.frame(x = x, y = y))

```