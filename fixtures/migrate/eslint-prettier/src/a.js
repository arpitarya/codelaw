var count = 1;
let total = 2;
console.log(count);
console.error(total);
debugger;
// eslint-disable-next-line no-console -- the startup banner
console.log("ready");
export const same = count === total;
export { count, total };
