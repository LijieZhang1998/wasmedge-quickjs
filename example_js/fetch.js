import { fetch } from "http";

print("test haah");

fetch("https://google.com", { method: "GET" }).then((res) => {
  print("connected");
});
