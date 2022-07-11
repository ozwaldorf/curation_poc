import { curation } from "../declarations/curation";

document.querySelector("form").addEventListener("submit", async (e) => {
  e.preventDefault();
  const button = e.target.querySelector("button");

  const sort_key = document.getElementById("sort_key").value.toString();
  const last_index = document.getElementById("last_index").value;
  const count = Number(document.getElementById("count").value);
  const reverse = document.getElementById("reverse").checked;
  const base_filter = document.getElementById("base_filter").value;

  const base_list = base_filter.split(",");

  const traits = base_filter
    ? [
        base_list.map((base) => {
          return ["base", { TextContent: base.trim() }];
        }),
      ]
    : [];

  button.setAttribute("disabled", true);

  let list = document.getElementById("data");
  // clear the list
  while (list.firstChild) {
    list.removeChild(list.firstChild);
  }
  list.innerHTML += `<p>Loading...</p><hr>`;

  // Interact with foo actor, calling the greet method
  const resp = await curation.query({
    sort_key,
    reverse: [reverse],
    count: [count],
    last_index: last_index ? [Number(last_index)] : [],
    traits: traits,
  });

  console.log(resp);

  button.removeAttribute("disabled");

  list.innerHTML = `
  <p>total (in index): ${resp.total} | last_index: ${
    resp.last_index[0] ? resp.last_index : "none"
  } | error: ${resp.error[0] ? resp.error : "none"}</p>
  <hr>`;

  for (const i in resp.data) {
    const token = resp.data[i];
    const trait_list = () => {
      let row1 = "<tr>";
      let row2 = "<tr>";
      // build trait name row
      for (const trait of token.traits[0]) {
        if (trait[0] !== "location") {
          row1 += `<td>${trait[0]}</td>`;
          row2 += `<td>${trait[1].TextContent}</td>`;
        }
      }
      row1 += "</tr>";
      row2 += "</tr>";
      const trait_string = row1 + row2;

      console.log(trait_string);

      return trait_string;
    };

    const traits = Object.fromEntries(token.traits[0]);
    console.log(traits);

    list.innerHTML += `
    <div class="token">
      <div class="header">
          <p><small style="color: #333">(${i})</small> token ${token.id}<p>
      </div>
      <img src=${traits.location.TextContent}></img>
      <div class="container">
        <small>listing price: ${token.price[0] ? token.price : "none"}</small>
        <br>
        <small>sale price: ${
          token.last_sale[0] ? token.last_sale[0].price : "none"
        }</small>
        <br>
        <small>offer price: ${
          token.offers.length ? token.best_offer[0] : "no offers"
        }</small>
        <hr>
        <small>last listed: ${
          token.last_listing[0] ? Number(token.last_listing[0]) : "none"
        }</small>
        <br>
        <small>last offered: ${
          token.last_offer[0] ? Number(token.last_offer[0]) : "none"
        }</small>
        <br>
        <small>last sold: ${
          token.last_sale[0] ? Number(token.last_sale[0].time) : "none"
        }</small>
        <hr>
        <table><tbody>
          ${trait_list()}
        </tbody></table>
      </div>
    </div>
    `;
  }

  return false;
});
