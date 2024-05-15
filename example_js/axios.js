import { fetch } from "http";
import redaxios from "redaxios";

const axios = redaxios({ fetch, responseType: "json" });

axios
  .get("http://localhost:8000")
  .then((response) => {
    console.log("connected");
  })
  .catch((e) => {
    console.log("exception: ", e);
  });
