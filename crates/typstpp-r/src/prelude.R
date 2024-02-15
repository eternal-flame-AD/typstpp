hooks_typst <- function() {
    list(
        source = function(x, options) {
            paste0("#src[\n```r\n", x, "\n```\n]\n")
        },
        output = function(x, options) {
            paste0("```\n", x, "\n```\n")
        },
        warning = function(x, options) {
            paste0("#emoji.warning `", x, "`")
        },
        message = function(x, options) {
            paste0("#emoji.info `", x, "`")
        },
        error = function(x, options) {
            paste0("#emoji.crossmark `", x, "`")
        },
        inline = function(x, options) {
            paste0("`", x, "`")
        },
        chunk = function(x, options) {
            paste0(x, "\n")
        },
        plot = function(x, options) {
            # escape plot environments from kframe
            paste0("#image(\"", x, "\")")
        }
    )
}

knitr::opts_chunk$set(dev = "svg")
knitr::knit_hooks$set(hooks_typst())
