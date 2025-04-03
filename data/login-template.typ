// Page heading
#set page(
  header: [
    #set text(style: "italic")
    #title
    #block(line(length: 100%, stroke: 0.5pt), above: 0.6em)
  ]
)

#let codeblock(content) = {
  box(
    inset: .75em,
    width: 100%,
    raw(content)
  )
};

#let logins(logins) = grid(
  columns: (1fr),
  stroke: (paint: black, thickness: 1pt, dash: "dashed"),
  ..logins.map(l => {
    box(inset: 1em)[
      #grid(
        columns: (1fr, 1fr),
        column-gutter: 1em,
        row-gutter: .5em,
        strong[Username],
        strong[Password],
        raw(l.username),
        raw(l.password),
      )
    ]
  })
)


= Host Logins
#logins(hosts)

= Competitor Logins
#logins(competitors)
