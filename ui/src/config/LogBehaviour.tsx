// import { LogBehaviour } from "$rs/LogBehaviour"
// import { For, createComputed, createEffect, createSignal } from "solid-js"
// import { LogDestination, TuplifyUnion, UnionToIntersection, invoke } from "../overrides";
// import { LogLevel } from "$rs/LogLevel";
// import { AiOutlineFolder } from "solid-icons/ai";

// const logDestinations: TuplifyUnion<LogDestination> = ["UIToast", "File", "StdOut", "StdErr"];
// const logLevels: TuplifyUnion<LogLevel> = ["Debug", "Info", "Error"];


// export default (props: { existing?: LogBehaviour }) => {
//     const [behaviour, changeBehaviour] = createSignal(props.existing ?? { New: undefined });
//     const dest = () => Object.keys(behaviour())[0] as LogDestination;
//     createEffect(() => console.log(behaviour()));
//     const changeDestination = (destination: LogDestination) => {
//         if (destination === "File") changeBehaviour({ [destination]: ["", logLevels[0]] })
//         else changeBehaviour({[destination]: "Debug"});
//     };
//     return <div>
//         <select class="join-item select" value={dest()} onInput={e => changeDestination(e.target.value as LogDestination)}>
//             <For each={logDestinations}>{dest =>
//                 <option value={dest}>{dest}</option>
//             }</For>
//         </select>
//         {
//             (() => {
//                 const b = behaviour();
//                 if ("File" in b)
//                     return <>
//                         {/* <button class="join-item btn btn-primary" onClick={async () => {
//                             console.log(await invoke("select_file"), "Info");
//                         }}><AiOutlineFolder /></button> */}
//                         <select class="join-item select" value={b.File[1]} onInput={e => (behaviour() as any)[dest()] = e.target.value}>
//                             <For each={logLevels}>{dest =>
//                                 <option value={dest}>{dest}</option>
//                             }</For>
//                         </select>
//                     </>
//                 else
//                     return <select class="join-item select" value={(b as any)[dest()]} onInput={e => (behaviour() as any)[dest()] = e.target.value}>
//                         <For each={logLevels}>{dest =>
//                             <option value={dest}>{dest}</option>
//                         }</For>
//                     </select>
//             })()
//         }
//     </div>
// }