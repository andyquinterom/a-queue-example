system.time({
  for (color in colors()) {
    Sys.sleep(1);
    httr2::request("http://localhost:3000") |>
      httr2::req_method("GET") |>
      httr2::req_url_query(arg = color) |>
      httr2::req_perform()
  }
})
