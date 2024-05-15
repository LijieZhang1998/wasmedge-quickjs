import * as std from "std";

//This file is not working because std.popen() is not working. I'm asking it in https://github.com/second-state/wasmedge-quickjs/issues/139
function fetchPolyfill(resource, init) {
  print("fetchPolyfill====================");

  init = init || {
    method: "GET",
    headers: null,
    body: null,
  };

  // method is always in upper case
  init.method = init.method.toUpperCase();

  // curl command
  let curlCmd = `curl -s -X${init.method.toUpperCase()} "${resource}"`;

  if (init.headers != null && Object.keys(init.headers).length > 0) {
    curlCmd = `${curlCmd} ${Object.entries(init.headers)
      .map((n) => `-H '${n[0]}: ${n[1]}'`)
      .join(" ")}`;
  }
  print("fetchPolyfill====================2");
  if (init.method != "GET") {
    let body = init.body;

    if (typeof body != "string") {
      body = JSON.stringify(body);
    }

    curlCmd = `${curlCmd} -d '${body}'`;
  }

  // exec curl command in subprocess
  const spErr = {};
  print("fetchPolyfill====================3");
  const sp = std.popen(curlCmd, "r", spErr);
  print("popen====================", sp);
  const curlOutput = sp.readAsString();
  const responseUrl = resource;
  const responseHeaders = {}; // FIXME: to be implemented
  let responseOk = true; // FIXME: to be implemented
  let responseStatus = 200; // FIXME: to be implemented

  const p = new Promise((resolve, reject) => {
    const response = {
      headers: responseHeaders,
      ok: responseOk,
      url: responseUrl,
      type: "json",
      text: () => curlOutput,
      json: () => JSON.parse(curlOutput),
    };

    resolve(response);
  });

  return p;
}

export default fetchPolyfill;
