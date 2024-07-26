#let src(content) = {
  block(
    fill: rgb("#ececec"), 
    inset: 1em,
    breakable: true)[
      #content
  ]
}
#align(center)[
  *Typstpp Demo*
]

Load some libraries:

#src[
```r
library(tidyverse)
```
]



Then make a plot:

#src[
```r
plot(iris)
```
]
#image("figures/typstpp-chunk-1-unnamed-chunk-1-1.svg")

Then try some Haskell:

#src[
```hs
:{
  fib :: Int -> Int
  fib 0 = 0
  fib 1 = 1
  fib n = fib (n-1) + fib (n-2)
:}

map fib [0..10]
```
]
```
[0,1,1,2,3,5,8,13,21,34,55]

```

Then make a table:

#src[
```r
knitr::kable(head(iris))
```
]


#table(
columns: (auto, auto, auto, auto, auto),
align: (right, right, right, right, left),
[Sepal.Length],[Sepal.Width],[Petal.Length],[Petal.Width],[Species],
[5.1],[3.5],[1.4],[0.2],[setosa],[4.9],[3.0],[1.4],[0.2],[setosa],[4.7],[3.2],[1.3],[0.2],[setosa],[4.6],[3.1],[1.5],[0.2],[setosa],[5.0],[3.6],[1.4],[0.2],[setosa],[5.4],[3.9],[1.7],[0.4],[setosa],
)



Mix some code, plots and tables in the same chunk:

#src[
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
```
]
#image("figures/typstpp-chunk-3-unnamed-chunk-1-1.svg")#src[
```r
print("↑ base R plot ↓ ggplot2 plot")
```
]
```
## [1] "↑ base R plot ↓ ggplot2 plot"

```
#src[
```r
ggplot(data.frame(x = x, y = y), aes(x, y)) +
  geom_line() +
  labs(title = "Factorial function", x = "x", y = "y")
```
]
#image("figures/typstpp-chunk-3-unnamed-chunk-1-2.svg")#src[
```r
knitr::kable(data.frame(x = x, y = y))
```
]


#table(
columns: (auto, auto),
align: (right, right),
[x],[y],
[1],[1],[2],[2],[3],[6],[4],[24],[5],[120],[6],[720],[7],[5040],[8],[40320],[9],[362880],[10],[3628800],
)


