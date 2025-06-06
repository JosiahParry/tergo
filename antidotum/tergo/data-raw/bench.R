# Run on Mac M1
# Dependencies: git2r, styler, tergo, microbenchmark, ggplot2
# install.packages(c("git2r", "tergo", "styler", "microbenchmark", "ggplot2"))
require(git2r)
require(tergo)
require(styler)
require(microbenchmark)
require(ggplot2)

tempd <- tempdir()
bench_path <- file.path(tempd, "bench")
dir.create(bench_path)

pacs <- c(
  "dplyr" = "https://github.com/tidyverse/dplyr",
  "ggplot2" = "https://github.com/tidyverse/ggplot2",
  "shiny" = "https://github.com/rstudio/shiny"
)

results <- list()
results2 <- list()

for (pac in names(pacs)) {
  pac_path <- file.path(bench_path, pac)
  pac_path_styler <- file.path(pac_path, "styler")
  pac_path_tergo <- file.path(pac_path, "tergo")

  # Independent so second styling is no rerunning
  git2r::clone(pacs[pac], pac_path_styler, progress = FALSE)
  git2r::clone(pacs[pac], pac_path_tergo, progress = FALSE)

  results[[pac]] <- microbenchmark::microbenchmark(
    styler = {
      styler::style_pkg(pac_path_styler, filetype = "R", dry = "off")
    },
    tergo = {
      tergo::style_pkg(pac_path_tergo)
    },
    times = 1,
    unit = "s"
  )

  results2[[pac]] <- microbenchmark::microbenchmark(
    styler = {
      styler::style_pkg(pac_path_styler, filetype = "R", dry = "off")
    },
    tergo = {
      tergo::style_pkg(pac_path_tergo)
    },
    times = 1,
    unit = "s"
  )
}

results_df <- do.call(
  rbind,
  lapply(names(results), function(x) cbind(summary(results[[x]])[, c("expr", "median")], package = x))
)

gg <- ggplot2::ggplot(results_df, ggplot2::aes(x = package, y = median)) +
  ggplot2::geom_bar(ggplot2::aes(fill = expr), stat = "identity", position = "dodge") +
  ggplot2::ggtitle("First Run\nSeconds to Style the Whole package x\nstyler vs tergo") +
  ggplot2::coord_flip() +
  ggplot2::labs(y = "seconds")

ggplot2::ggsave("man/figures/first_run.png", gg)

results2_df <- do.call(
  rbind,
  lapply(names(results), function(x) cbind(summary(results2[[x]])[, c("expr", "median")], package = x))
)

gg2 <- ggplot2::ggplot(results2_df, ggplot2::aes(x = package, y = median)) +
  ggplot2::geom_bar(ggplot2::aes(fill = expr), stat = "identity", position = "dodge") +
  ggplot2::coord_flip() +
  ggplot2::ggtitle("Second and Subsequent Runs\nSeconds to Style the Whole package x\nstyler vs tergo") +
  ggplot2::labs(y = "seconds")

ggplot2::ggsave("man/figures/second_run.png", gg2)

