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
#image("figure/unnamed-chunk-1-1.svg")

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


