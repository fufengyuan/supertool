#!/usr/bin/env node
import { createRequire } from 'module'; const require = createRequire(import.meta.url);
var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __require = /* @__PURE__ */ ((x2) => typeof require !== "undefined" ? require : typeof Proxy !== "undefined" ? new Proxy(x2, {
  get: (a, b2) => (typeof require !== "undefined" ? require : a)[b2]
}) : x2)(function(x2) {
  if (typeof require !== "undefined") return require.apply(this, arguments);
  throw Error('Dynamic require of "' + x2 + '" is not supported');
});
var __esm = (fn, res) => function __init() {
  return fn && (res = (0, fn[__getOwnPropNames(fn)[0]])(fn = 0)), res;
};
var __commonJS = (cb, mod) => function __require2() {
  return mod || (0, cb[__getOwnPropNames(cb)[0]])((mod = { exports: {} }).exports, mod), mod.exports;
};
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));

// node_modules/commander/lib/error.js
var require_error = __commonJS({
  "node_modules/commander/lib/error.js"(exports) {
    var CommanderError2 = class extends Error {
      /**
       * Constructs the CommanderError class
       * @param {number} exitCode suggested exit code which could be used with process.exit
       * @param {string} code an id string representing the error
       * @param {string} message human-readable description of the error
       */
      constructor(exitCode, code, message) {
        super(message);
        Error.captureStackTrace(this, this.constructor);
        this.name = this.constructor.name;
        this.code = code;
        this.exitCode = exitCode;
        this.nestedError = void 0;
      }
    };
    var InvalidArgumentError2 = class extends CommanderError2 {
      /**
       * Constructs the InvalidArgumentError class
       * @param {string} [message] explanation of why argument is invalid
       */
      constructor(message) {
        super(1, "commander.invalidArgument", message);
        Error.captureStackTrace(this, this.constructor);
        this.name = this.constructor.name;
      }
    };
    exports.CommanderError = CommanderError2;
    exports.InvalidArgumentError = InvalidArgumentError2;
  }
});

// node_modules/commander/lib/argument.js
var require_argument = __commonJS({
  "node_modules/commander/lib/argument.js"(exports) {
    var { InvalidArgumentError: InvalidArgumentError2 } = require_error();
    var Argument2 = class {
      /**
       * Initialize a new command argument with the given name and description.
       * The default is that the argument is required, and you can explicitly
       * indicate this with <> around the name. Put [] around the name for an optional argument.
       *
       * @param {string} name
       * @param {string} [description]
       */
      constructor(name, description) {
        this.description = description || "";
        this.variadic = false;
        this.parseArg = void 0;
        this.defaultValue = void 0;
        this.defaultValueDescription = void 0;
        this.argChoices = void 0;
        switch (name[0]) {
          case "<":
            this.required = true;
            this._name = name.slice(1, -1);
            break;
          case "[":
            this.required = false;
            this._name = name.slice(1, -1);
            break;
          default:
            this.required = true;
            this._name = name;
            break;
        }
        if (this._name.endsWith("...")) {
          this.variadic = true;
          this._name = this._name.slice(0, -3);
        }
      }
      /**
       * Return argument name.
       *
       * @return {string}
       */
      name() {
        return this._name;
      }
      /**
       * @package
       */
      _collectValue(value, previous) {
        if (previous === this.defaultValue || !Array.isArray(previous)) {
          return [value];
        }
        previous.push(value);
        return previous;
      }
      /**
       * Set the default value, and optionally supply the description to be displayed in the help.
       *
       * @param {*} value
       * @param {string} [description]
       * @return {Argument}
       */
      default(value, description) {
        this.defaultValue = value;
        this.defaultValueDescription = description;
        return this;
      }
      /**
       * Set the custom handler for processing CLI command arguments into argument values.
       *
       * @param {Function} [fn]
       * @return {Argument}
       */
      argParser(fn) {
        this.parseArg = fn;
        return this;
      }
      /**
       * Only allow argument value to be one of choices.
       *
       * @param {string[]} values
       * @return {Argument}
       */
      choices(values) {
        this.argChoices = values.slice();
        this.parseArg = (arg, previous) => {
          if (!this.argChoices.includes(arg)) {
            throw new InvalidArgumentError2(
              `Allowed choices are ${this.argChoices.join(", ")}.`
            );
          }
          if (this.variadic) {
            return this._collectValue(arg, previous);
          }
          return arg;
        };
        return this;
      }
      /**
       * Make argument required.
       *
       * @returns {Argument}
       */
      argRequired() {
        this.required = true;
        return this;
      }
      /**
       * Make argument optional.
       *
       * @returns {Argument}
       */
      argOptional() {
        this.required = false;
        return this;
      }
    };
    function humanReadableArgName(arg) {
      const nameOutput = arg.name() + (arg.variadic === true ? "..." : "");
      return arg.required ? "<" + nameOutput + ">" : "[" + nameOutput + "]";
    }
    exports.Argument = Argument2;
    exports.humanReadableArgName = humanReadableArgName;
  }
});

// node_modules/commander/lib/help.js
var require_help = __commonJS({
  "node_modules/commander/lib/help.js"(exports) {
    var { humanReadableArgName } = require_argument();
    var Help2 = class {
      constructor() {
        this.helpWidth = void 0;
        this.minWidthToWrap = 40;
        this.sortSubcommands = false;
        this.sortOptions = false;
        this.showGlobalOptions = false;
      }
      /**
       * prepareContext is called by Commander after applying overrides from `Command.configureHelp()`
       * and just before calling `formatHelp()`.
       *
       * Commander just uses the helpWidth and the rest is provided for optional use by more complex subclasses.
       *
       * @param {{ error?: boolean, helpWidth?: number, outputHasColors?: boolean }} contextOptions
       */
      prepareContext(contextOptions) {
        this.helpWidth = this.helpWidth ?? contextOptions.helpWidth ?? 80;
      }
      /**
       * Get an array of the visible subcommands. Includes a placeholder for the implicit help command, if there is one.
       *
       * @param {Command} cmd
       * @returns {Command[]}
       */
      visibleCommands(cmd) {
        const visibleCommands = cmd.commands.filter((cmd2) => !cmd2._hidden);
        const helpCommand = cmd._getHelpCommand();
        if (helpCommand && !helpCommand._hidden) {
          visibleCommands.push(helpCommand);
        }
        if (this.sortSubcommands) {
          visibleCommands.sort((a, b2) => {
            return a.name().localeCompare(b2.name());
          });
        }
        return visibleCommands;
      }
      /**
       * Compare options for sort.
       *
       * @param {Option} a
       * @param {Option} b
       * @returns {number}
       */
      compareOptions(a, b2) {
        const getSortKey = (option) => {
          return option.short ? option.short.replace(/^-/, "") : option.long.replace(/^--/, "");
        };
        return getSortKey(a).localeCompare(getSortKey(b2));
      }
      /**
       * Get an array of the visible options. Includes a placeholder for the implicit help option, if there is one.
       *
       * @param {Command} cmd
       * @returns {Option[]}
       */
      visibleOptions(cmd) {
        const visibleOptions = cmd.options.filter((option) => !option.hidden);
        const helpOption = cmd._getHelpOption();
        if (helpOption && !helpOption.hidden) {
          const removeShort = helpOption.short && cmd._findOption(helpOption.short);
          const removeLong = helpOption.long && cmd._findOption(helpOption.long);
          if (!removeShort && !removeLong) {
            visibleOptions.push(helpOption);
          } else if (helpOption.long && !removeLong) {
            visibleOptions.push(
              cmd.createOption(helpOption.long, helpOption.description)
            );
          } else if (helpOption.short && !removeShort) {
            visibleOptions.push(
              cmd.createOption(helpOption.short, helpOption.description)
            );
          }
        }
        if (this.sortOptions) {
          visibleOptions.sort(this.compareOptions);
        }
        return visibleOptions;
      }
      /**
       * Get an array of the visible global options. (Not including help.)
       *
       * @param {Command} cmd
       * @returns {Option[]}
       */
      visibleGlobalOptions(cmd) {
        if (!this.showGlobalOptions) return [];
        const globalOptions = [];
        for (let ancestorCmd = cmd.parent; ancestorCmd; ancestorCmd = ancestorCmd.parent) {
          const visibleOptions = ancestorCmd.options.filter(
            (option) => !option.hidden
          );
          globalOptions.push(...visibleOptions);
        }
        if (this.sortOptions) {
          globalOptions.sort(this.compareOptions);
        }
        return globalOptions;
      }
      /**
       * Get an array of the arguments if any have a description.
       *
       * @param {Command} cmd
       * @returns {Argument[]}
       */
      visibleArguments(cmd) {
        if (cmd._argsDescription) {
          cmd.registeredArguments.forEach((argument) => {
            argument.description = argument.description || cmd._argsDescription[argument.name()] || "";
          });
        }
        if (cmd.registeredArguments.find((argument) => argument.description)) {
          return cmd.registeredArguments;
        }
        return [];
      }
      /**
       * Get the command term to show in the list of subcommands.
       *
       * @param {Command} cmd
       * @returns {string}
       */
      subcommandTerm(cmd) {
        const args = cmd.registeredArguments.map((arg) => humanReadableArgName(arg)).join(" ");
        return cmd._name + (cmd._aliases[0] ? "|" + cmd._aliases[0] : "") + (cmd.options.length ? " [options]" : "") + // simplistic check for non-help option
        (args ? " " + args : "");
      }
      /**
       * Get the option term to show in the list of options.
       *
       * @param {Option} option
       * @returns {string}
       */
      optionTerm(option) {
        return option.flags;
      }
      /**
       * Get the argument term to show in the list of arguments.
       *
       * @param {Argument} argument
       * @returns {string}
       */
      argumentTerm(argument) {
        return argument.name();
      }
      /**
       * Get the longest command term length.
       *
       * @param {Command} cmd
       * @param {Help} helper
       * @returns {number}
       */
      longestSubcommandTermLength(cmd, helper) {
        return helper.visibleCommands(cmd).reduce((max, command) => {
          return Math.max(
            max,
            this.displayWidth(
              helper.styleSubcommandTerm(helper.subcommandTerm(command))
            )
          );
        }, 0);
      }
      /**
       * Get the longest option term length.
       *
       * @param {Command} cmd
       * @param {Help} helper
       * @returns {number}
       */
      longestOptionTermLength(cmd, helper) {
        return helper.visibleOptions(cmd).reduce((max, option) => {
          return Math.max(
            max,
            this.displayWidth(helper.styleOptionTerm(helper.optionTerm(option)))
          );
        }, 0);
      }
      /**
       * Get the longest global option term length.
       *
       * @param {Command} cmd
       * @param {Help} helper
       * @returns {number}
       */
      longestGlobalOptionTermLength(cmd, helper) {
        return helper.visibleGlobalOptions(cmd).reduce((max, option) => {
          return Math.max(
            max,
            this.displayWidth(helper.styleOptionTerm(helper.optionTerm(option)))
          );
        }, 0);
      }
      /**
       * Get the longest argument term length.
       *
       * @param {Command} cmd
       * @param {Help} helper
       * @returns {number}
       */
      longestArgumentTermLength(cmd, helper) {
        return helper.visibleArguments(cmd).reduce((max, argument) => {
          return Math.max(
            max,
            this.displayWidth(
              helper.styleArgumentTerm(helper.argumentTerm(argument))
            )
          );
        }, 0);
      }
      /**
       * Get the command usage to be displayed at the top of the built-in help.
       *
       * @param {Command} cmd
       * @returns {string}
       */
      commandUsage(cmd) {
        let cmdName = cmd._name;
        if (cmd._aliases[0]) {
          cmdName = cmdName + "|" + cmd._aliases[0];
        }
        let ancestorCmdNames = "";
        for (let ancestorCmd = cmd.parent; ancestorCmd; ancestorCmd = ancestorCmd.parent) {
          ancestorCmdNames = ancestorCmd.name() + " " + ancestorCmdNames;
        }
        return ancestorCmdNames + cmdName + " " + cmd.usage();
      }
      /**
       * Get the description for the command.
       *
       * @param {Command} cmd
       * @returns {string}
       */
      commandDescription(cmd) {
        return cmd.description();
      }
      /**
       * Get the subcommand summary to show in the list of subcommands.
       * (Fallback to description for backwards compatibility.)
       *
       * @param {Command} cmd
       * @returns {string}
       */
      subcommandDescription(cmd) {
        return cmd.summary() || cmd.description();
      }
      /**
       * Get the option description to show in the list of options.
       *
       * @param {Option} option
       * @return {string}
       */
      optionDescription(option) {
        const extraInfo = [];
        if (option.argChoices) {
          extraInfo.push(
            // use stringify to match the display of the default value
            `choices: ${option.argChoices.map((choice) => JSON.stringify(choice)).join(", ")}`
          );
        }
        if (option.defaultValue !== void 0) {
          const showDefault = option.required || option.optional || option.isBoolean() && typeof option.defaultValue === "boolean";
          if (showDefault) {
            extraInfo.push(
              `default: ${option.defaultValueDescription || JSON.stringify(option.defaultValue)}`
            );
          }
        }
        if (option.presetArg !== void 0 && option.optional) {
          extraInfo.push(`preset: ${JSON.stringify(option.presetArg)}`);
        }
        if (option.envVar !== void 0) {
          extraInfo.push(`env: ${option.envVar}`);
        }
        if (extraInfo.length > 0) {
          const extraDescription = `(${extraInfo.join(", ")})`;
          if (option.description) {
            return `${option.description} ${extraDescription}`;
          }
          return extraDescription;
        }
        return option.description;
      }
      /**
       * Get the argument description to show in the list of arguments.
       *
       * @param {Argument} argument
       * @return {string}
       */
      argumentDescription(argument) {
        const extraInfo = [];
        if (argument.argChoices) {
          extraInfo.push(
            // use stringify to match the display of the default value
            `choices: ${argument.argChoices.map((choice) => JSON.stringify(choice)).join(", ")}`
          );
        }
        if (argument.defaultValue !== void 0) {
          extraInfo.push(
            `default: ${argument.defaultValueDescription || JSON.stringify(argument.defaultValue)}`
          );
        }
        if (extraInfo.length > 0) {
          const extraDescription = `(${extraInfo.join(", ")})`;
          if (argument.description) {
            return `${argument.description} ${extraDescription}`;
          }
          return extraDescription;
        }
        return argument.description;
      }
      /**
       * Format a list of items, given a heading and an array of formatted items.
       *
       * @param {string} heading
       * @param {string[]} items
       * @param {Help} helper
       * @returns string[]
       */
      formatItemList(heading, items, helper) {
        if (items.length === 0) return [];
        return [helper.styleTitle(heading), ...items, ""];
      }
      /**
       * Group items by their help group heading.
       *
       * @param {Command[] | Option[]} unsortedItems
       * @param {Command[] | Option[]} visibleItems
       * @param {Function} getGroup
       * @returns {Map<string, Command[] | Option[]>}
       */
      groupItems(unsortedItems, visibleItems, getGroup) {
        const result = /* @__PURE__ */ new Map();
        unsortedItems.forEach((item) => {
          const group = getGroup(item);
          if (!result.has(group)) result.set(group, []);
        });
        visibleItems.forEach((item) => {
          const group = getGroup(item);
          if (!result.has(group)) {
            result.set(group, []);
          }
          result.get(group).push(item);
        });
        return result;
      }
      /**
       * Generate the built-in help text.
       *
       * @param {Command} cmd
       * @param {Help} helper
       * @returns {string}
       */
      formatHelp(cmd, helper) {
        const termWidth = helper.padWidth(cmd, helper);
        const helpWidth = helper.helpWidth ?? 80;
        function callFormatItem(term, description) {
          return helper.formatItem(term, termWidth, description, helper);
        }
        let output = [
          `${helper.styleTitle("Usage:")} ${helper.styleUsage(helper.commandUsage(cmd))}`,
          ""
        ];
        const commandDescription = helper.commandDescription(cmd);
        if (commandDescription.length > 0) {
          output = output.concat([
            helper.boxWrap(
              helper.styleCommandDescription(commandDescription),
              helpWidth
            ),
            ""
          ]);
        }
        const argumentList = helper.visibleArguments(cmd).map((argument) => {
          return callFormatItem(
            helper.styleArgumentTerm(helper.argumentTerm(argument)),
            helper.styleArgumentDescription(helper.argumentDescription(argument))
          );
        });
        output = output.concat(
          this.formatItemList("Arguments:", argumentList, helper)
        );
        const optionGroups = this.groupItems(
          cmd.options,
          helper.visibleOptions(cmd),
          (option) => option.helpGroupHeading ?? "Options:"
        );
        optionGroups.forEach((options, group) => {
          const optionList = options.map((option) => {
            return callFormatItem(
              helper.styleOptionTerm(helper.optionTerm(option)),
              helper.styleOptionDescription(helper.optionDescription(option))
            );
          });
          output = output.concat(this.formatItemList(group, optionList, helper));
        });
        if (helper.showGlobalOptions) {
          const globalOptionList = helper.visibleGlobalOptions(cmd).map((option) => {
            return callFormatItem(
              helper.styleOptionTerm(helper.optionTerm(option)),
              helper.styleOptionDescription(helper.optionDescription(option))
            );
          });
          output = output.concat(
            this.formatItemList("Global Options:", globalOptionList, helper)
          );
        }
        const commandGroups = this.groupItems(
          cmd.commands,
          helper.visibleCommands(cmd),
          (sub) => sub.helpGroup() || "Commands:"
        );
        commandGroups.forEach((commands, group) => {
          const commandList = commands.map((sub) => {
            return callFormatItem(
              helper.styleSubcommandTerm(helper.subcommandTerm(sub)),
              helper.styleSubcommandDescription(helper.subcommandDescription(sub))
            );
          });
          output = output.concat(this.formatItemList(group, commandList, helper));
        });
        return output.join("\n");
      }
      /**
       * Return display width of string, ignoring ANSI escape sequences. Used in padding and wrapping calculations.
       *
       * @param {string} str
       * @returns {number}
       */
      displayWidth(str) {
        return stripColor(str).length;
      }
      /**
       * Style the title for displaying in the help. Called with 'Usage:', 'Options:', etc.
       *
       * @param {string} str
       * @returns {string}
       */
      styleTitle(str) {
        return str;
      }
      styleUsage(str) {
        return str.split(" ").map((word) => {
          if (word === "[options]") return this.styleOptionText(word);
          if (word === "[command]") return this.styleSubcommandText(word);
          if (word[0] === "[" || word[0] === "<")
            return this.styleArgumentText(word);
          return this.styleCommandText(word);
        }).join(" ");
      }
      styleCommandDescription(str) {
        return this.styleDescriptionText(str);
      }
      styleOptionDescription(str) {
        return this.styleDescriptionText(str);
      }
      styleSubcommandDescription(str) {
        return this.styleDescriptionText(str);
      }
      styleArgumentDescription(str) {
        return this.styleDescriptionText(str);
      }
      styleDescriptionText(str) {
        return str;
      }
      styleOptionTerm(str) {
        return this.styleOptionText(str);
      }
      styleSubcommandTerm(str) {
        return str.split(" ").map((word) => {
          if (word === "[options]") return this.styleOptionText(word);
          if (word[0] === "[" || word[0] === "<")
            return this.styleArgumentText(word);
          return this.styleSubcommandText(word);
        }).join(" ");
      }
      styleArgumentTerm(str) {
        return this.styleArgumentText(str);
      }
      styleOptionText(str) {
        return str;
      }
      styleArgumentText(str) {
        return str;
      }
      styleSubcommandText(str) {
        return str;
      }
      styleCommandText(str) {
        return str;
      }
      /**
       * Calculate the pad width from the maximum term length.
       *
       * @param {Command} cmd
       * @param {Help} helper
       * @returns {number}
       */
      padWidth(cmd, helper) {
        return Math.max(
          helper.longestOptionTermLength(cmd, helper),
          helper.longestGlobalOptionTermLength(cmd, helper),
          helper.longestSubcommandTermLength(cmd, helper),
          helper.longestArgumentTermLength(cmd, helper)
        );
      }
      /**
       * Detect manually wrapped and indented strings by checking for line break followed by whitespace.
       *
       * @param {string} str
       * @returns {boolean}
       */
      preformatted(str) {
        return /\n[^\S\r\n]/.test(str);
      }
      /**
       * Format the "item", which consists of a term and description. Pad the term and wrap the description, indenting the following lines.
       *
       * So "TTT", 5, "DDD DDDD DD DDD" might be formatted for this.helpWidth=17 like so:
       *   TTT  DDD DDDD
       *        DD DDD
       *
       * @param {string} term
       * @param {number} termWidth
       * @param {string} description
       * @param {Help} helper
       * @returns {string}
       */
      formatItem(term, termWidth, description, helper) {
        const itemIndent = 2;
        const itemIndentStr = " ".repeat(itemIndent);
        if (!description) return itemIndentStr + term;
        const paddedTerm = term.padEnd(
          termWidth + term.length - helper.displayWidth(term)
        );
        const spacerWidth = 2;
        const helpWidth = this.helpWidth ?? 80;
        const remainingWidth = helpWidth - termWidth - spacerWidth - itemIndent;
        let formattedDescription;
        if (remainingWidth < this.minWidthToWrap || helper.preformatted(description)) {
          formattedDescription = description;
        } else {
          const wrappedDescription = helper.boxWrap(description, remainingWidth);
          formattedDescription = wrappedDescription.replace(
            /\n/g,
            "\n" + " ".repeat(termWidth + spacerWidth)
          );
        }
        return itemIndentStr + paddedTerm + " ".repeat(spacerWidth) + formattedDescription.replace(/\n/g, `
${itemIndentStr}`);
      }
      /**
       * Wrap a string at whitespace, preserving existing line breaks.
       * Wrapping is skipped if the width is less than `minWidthToWrap`.
       *
       * @param {string} str
       * @param {number} width
       * @returns {string}
       */
      boxWrap(str, width) {
        if (width < this.minWidthToWrap) return str;
        const rawLines = str.split(/\r\n|\n/);
        const chunkPattern = /[\s]*[^\s]+/g;
        const wrappedLines = [];
        rawLines.forEach((line) => {
          const chunks = line.match(chunkPattern);
          if (chunks === null) {
            wrappedLines.push("");
            return;
          }
          let sumChunks = [chunks.shift()];
          let sumWidth = this.displayWidth(sumChunks[0]);
          chunks.forEach((chunk) => {
            const visibleWidth = this.displayWidth(chunk);
            if (sumWidth + visibleWidth <= width) {
              sumChunks.push(chunk);
              sumWidth += visibleWidth;
              return;
            }
            wrappedLines.push(sumChunks.join(""));
            const nextChunk = chunk.trimStart();
            sumChunks = [nextChunk];
            sumWidth = this.displayWidth(nextChunk);
          });
          wrappedLines.push(sumChunks.join(""));
        });
        return wrappedLines.join("\n");
      }
    };
    function stripColor(str) {
      const sgrPattern = /\x1b\[\d*(;\d*)*m/g;
      return str.replace(sgrPattern, "");
    }
    exports.Help = Help2;
    exports.stripColor = stripColor;
  }
});

// node_modules/commander/lib/option.js
var require_option = __commonJS({
  "node_modules/commander/lib/option.js"(exports) {
    var { InvalidArgumentError: InvalidArgumentError2 } = require_error();
    var Option2 = class {
      /**
       * Initialize a new `Option` with the given `flags` and `description`.
       *
       * @param {string} flags
       * @param {string} [description]
       */
      constructor(flags, description) {
        this.flags = flags;
        this.description = description || "";
        this.required = flags.includes("<");
        this.optional = flags.includes("[");
        this.variadic = /\w\.\.\.[>\]]$/.test(flags);
        this.mandatory = false;
        const optionFlags = splitOptionFlags(flags);
        this.short = optionFlags.shortFlag;
        this.long = optionFlags.longFlag;
        this.negate = false;
        if (this.long) {
          this.negate = this.long.startsWith("--no-");
        }
        this.defaultValue = void 0;
        this.defaultValueDescription = void 0;
        this.presetArg = void 0;
        this.envVar = void 0;
        this.parseArg = void 0;
        this.hidden = false;
        this.argChoices = void 0;
        this.conflictsWith = [];
        this.implied = void 0;
        this.helpGroupHeading = void 0;
      }
      /**
       * Set the default value, and optionally supply the description to be displayed in the help.
       *
       * @param {*} value
       * @param {string} [description]
       * @return {Option}
       */
      default(value, description) {
        this.defaultValue = value;
        this.defaultValueDescription = description;
        return this;
      }
      /**
       * Preset to use when option used without option-argument, especially optional but also boolean and negated.
       * The custom processing (parseArg) is called.
       *
       * @example
       * new Option('--color').default('GREYSCALE').preset('RGB');
       * new Option('--donate [amount]').preset('20').argParser(parseFloat);
       *
       * @param {*} arg
       * @return {Option}
       */
      preset(arg) {
        this.presetArg = arg;
        return this;
      }
      /**
       * Add option name(s) that conflict with this option.
       * An error will be displayed if conflicting options are found during parsing.
       *
       * @example
       * new Option('--rgb').conflicts('cmyk');
       * new Option('--js').conflicts(['ts', 'jsx']);
       *
       * @param {(string | string[])} names
       * @return {Option}
       */
      conflicts(names) {
        this.conflictsWith = this.conflictsWith.concat(names);
        return this;
      }
      /**
       * Specify implied option values for when this option is set and the implied options are not.
       *
       * The custom processing (parseArg) is not called on the implied values.
       *
       * @example
       * program
       *   .addOption(new Option('--log', 'write logging information to file'))
       *   .addOption(new Option('--trace', 'log extra details').implies({ log: 'trace.txt' }));
       *
       * @param {object} impliedOptionValues
       * @return {Option}
       */
      implies(impliedOptionValues) {
        let newImplied = impliedOptionValues;
        if (typeof impliedOptionValues === "string") {
          newImplied = { [impliedOptionValues]: true };
        }
        this.implied = Object.assign(this.implied || {}, newImplied);
        return this;
      }
      /**
       * Set environment variable to check for option value.
       *
       * An environment variable is only used if when processed the current option value is
       * undefined, or the source of the current value is 'default' or 'config' or 'env'.
       *
       * @param {string} name
       * @return {Option}
       */
      env(name) {
        this.envVar = name;
        return this;
      }
      /**
       * Set the custom handler for processing CLI option arguments into option values.
       *
       * @param {Function} [fn]
       * @return {Option}
       */
      argParser(fn) {
        this.parseArg = fn;
        return this;
      }
      /**
       * Whether the option is mandatory and must have a value after parsing.
       *
       * @param {boolean} [mandatory=true]
       * @return {Option}
       */
      makeOptionMandatory(mandatory = true) {
        this.mandatory = !!mandatory;
        return this;
      }
      /**
       * Hide option in help.
       *
       * @param {boolean} [hide=true]
       * @return {Option}
       */
      hideHelp(hide = true) {
        this.hidden = !!hide;
        return this;
      }
      /**
       * @package
       */
      _collectValue(value, previous) {
        if (previous === this.defaultValue || !Array.isArray(previous)) {
          return [value];
        }
        previous.push(value);
        return previous;
      }
      /**
       * Only allow option value to be one of choices.
       *
       * @param {string[]} values
       * @return {Option}
       */
      choices(values) {
        this.argChoices = values.slice();
        this.parseArg = (arg, previous) => {
          if (!this.argChoices.includes(arg)) {
            throw new InvalidArgumentError2(
              `Allowed choices are ${this.argChoices.join(", ")}.`
            );
          }
          if (this.variadic) {
            return this._collectValue(arg, previous);
          }
          return arg;
        };
        return this;
      }
      /**
       * Return option name.
       *
       * @return {string}
       */
      name() {
        if (this.long) {
          return this.long.replace(/^--/, "");
        }
        return this.short.replace(/^-/, "");
      }
      /**
       * Return option name, in a camelcase format that can be used
       * as an object attribute key.
       *
       * @return {string}
       */
      attributeName() {
        if (this.negate) {
          return camelcase(this.name().replace(/^no-/, ""));
        }
        return camelcase(this.name());
      }
      /**
       * Set the help group heading.
       *
       * @param {string} heading
       * @return {Option}
       */
      helpGroup(heading) {
        this.helpGroupHeading = heading;
        return this;
      }
      /**
       * Check if `arg` matches the short or long flag.
       *
       * @param {string} arg
       * @return {boolean}
       * @package
       */
      is(arg) {
        return this.short === arg || this.long === arg;
      }
      /**
       * Return whether a boolean option.
       *
       * Options are one of boolean, negated, required argument, or optional argument.
       *
       * @return {boolean}
       * @package
       */
      isBoolean() {
        return !this.required && !this.optional && !this.negate;
      }
    };
    var DualOptions = class {
      /**
       * @param {Option[]} options
       */
      constructor(options) {
        this.positiveOptions = /* @__PURE__ */ new Map();
        this.negativeOptions = /* @__PURE__ */ new Map();
        this.dualOptions = /* @__PURE__ */ new Set();
        options.forEach((option) => {
          if (option.negate) {
            this.negativeOptions.set(option.attributeName(), option);
          } else {
            this.positiveOptions.set(option.attributeName(), option);
          }
        });
        this.negativeOptions.forEach((value, key) => {
          if (this.positiveOptions.has(key)) {
            this.dualOptions.add(key);
          }
        });
      }
      /**
       * Did the value come from the option, and not from possible matching dual option?
       *
       * @param {*} value
       * @param {Option} option
       * @returns {boolean}
       */
      valueFromOption(value, option) {
        const optionKey = option.attributeName();
        if (!this.dualOptions.has(optionKey)) return true;
        const preset = this.negativeOptions.get(optionKey).presetArg;
        const negativeValue = preset !== void 0 ? preset : false;
        return option.negate === (negativeValue === value);
      }
    };
    function camelcase(str) {
      return str.split("-").reduce((str2, word) => {
        return str2 + word[0].toUpperCase() + word.slice(1);
      });
    }
    function splitOptionFlags(flags) {
      let shortFlag;
      let longFlag;
      const shortFlagExp = /^-[^-]$/;
      const longFlagExp = /^--[^-]/;
      const flagParts = flags.split(/[ |,]+/).concat("guard");
      if (shortFlagExp.test(flagParts[0])) shortFlag = flagParts.shift();
      if (longFlagExp.test(flagParts[0])) longFlag = flagParts.shift();
      if (!shortFlag && shortFlagExp.test(flagParts[0]))
        shortFlag = flagParts.shift();
      if (!shortFlag && longFlagExp.test(flagParts[0])) {
        shortFlag = longFlag;
        longFlag = flagParts.shift();
      }
      if (flagParts[0].startsWith("-")) {
        const unsupportedFlag = flagParts[0];
        const baseError = `option creation failed due to '${unsupportedFlag}' in option flags '${flags}'`;
        if (/^-[^-][^-]/.test(unsupportedFlag))
          throw new Error(
            `${baseError}
- a short flag is a single dash and a single character
  - either use a single dash and a single character (for a short flag)
  - or use a double dash for a long option (and can have two, like '--ws, --workspace')`
          );
        if (shortFlagExp.test(unsupportedFlag))
          throw new Error(`${baseError}
- too many short flags`);
        if (longFlagExp.test(unsupportedFlag))
          throw new Error(`${baseError}
- too many long flags`);
        throw new Error(`${baseError}
- unrecognised flag format`);
      }
      if (shortFlag === void 0 && longFlag === void 0)
        throw new Error(
          `option creation failed due to no flags found in '${flags}'.`
        );
      return { shortFlag, longFlag };
    }
    exports.Option = Option2;
    exports.DualOptions = DualOptions;
  }
});

// node_modules/commander/lib/suggestSimilar.js
var require_suggestSimilar = __commonJS({
  "node_modules/commander/lib/suggestSimilar.js"(exports) {
    var maxDistance = 3;
    function editDistance(a, b2) {
      if (Math.abs(a.length - b2.length) > maxDistance)
        return Math.max(a.length, b2.length);
      const d = [];
      for (let i2 = 0; i2 <= a.length; i2++) {
        d[i2] = [i2];
      }
      for (let j2 = 0; j2 <= b2.length; j2++) {
        d[0][j2] = j2;
      }
      for (let j2 = 1; j2 <= b2.length; j2++) {
        for (let i2 = 1; i2 <= a.length; i2++) {
          let cost = 1;
          if (a[i2 - 1] === b2[j2 - 1]) {
            cost = 0;
          } else {
            cost = 1;
          }
          d[i2][j2] = Math.min(
            d[i2 - 1][j2] + 1,
            // deletion
            d[i2][j2 - 1] + 1,
            // insertion
            d[i2 - 1][j2 - 1] + cost
            // substitution
          );
          if (i2 > 1 && j2 > 1 && a[i2 - 1] === b2[j2 - 2] && a[i2 - 2] === b2[j2 - 1]) {
            d[i2][j2] = Math.min(d[i2][j2], d[i2 - 2][j2 - 2] + 1);
          }
        }
      }
      return d[a.length][b2.length];
    }
    function suggestSimilar(word, candidates) {
      if (!candidates || candidates.length === 0) return "";
      candidates = Array.from(new Set(candidates));
      const searchingOptions = word.startsWith("--");
      if (searchingOptions) {
        word = word.slice(2);
        candidates = candidates.map((candidate) => candidate.slice(2));
      }
      let similar = [];
      let bestDistance = maxDistance;
      const minSimilarity = 0.4;
      candidates.forEach((candidate) => {
        if (candidate.length <= 1) return;
        const distance = editDistance(word, candidate);
        const length = Math.max(word.length, candidate.length);
        const similarity = (length - distance) / length;
        if (similarity > minSimilarity) {
          if (distance < bestDistance) {
            bestDistance = distance;
            similar = [candidate];
          } else if (distance === bestDistance) {
            similar.push(candidate);
          }
        }
      });
      similar.sort((a, b2) => a.localeCompare(b2));
      if (searchingOptions) {
        similar = similar.map((candidate) => `--${candidate}`);
      }
      if (similar.length > 1) {
        return `
(Did you mean one of ${similar.join(", ")}?)`;
      }
      if (similar.length === 1) {
        return `
(Did you mean ${similar[0]}?)`;
      }
      return "";
    }
    exports.suggestSimilar = suggestSimilar;
  }
});

// node_modules/commander/lib/command.js
var require_command = __commonJS({
  "node_modules/commander/lib/command.js"(exports) {
    var EventEmitter2 = __require("node:events").EventEmitter;
    var childProcess = __require("node:child_process");
    var path2 = __require("node:path");
    var fs2 = __require("node:fs");
    var process3 = __require("node:process");
    var { Argument: Argument2, humanReadableArgName } = require_argument();
    var { CommanderError: CommanderError2 } = require_error();
    var { Help: Help2, stripColor } = require_help();
    var { Option: Option2, DualOptions } = require_option();
    var { suggestSimilar } = require_suggestSimilar();
    var Command2 = class _Command extends EventEmitter2 {
      /**
       * Initialize a new `Command`.
       *
       * @param {string} [name]
       */
      constructor(name) {
        super();
        this.commands = [];
        this.options = [];
        this.parent = null;
        this._allowUnknownOption = false;
        this._allowExcessArguments = false;
        this.registeredArguments = [];
        this._args = this.registeredArguments;
        this.args = [];
        this.rawArgs = [];
        this.processedArgs = [];
        this._scriptPath = null;
        this._name = name || "";
        this._optionValues = {};
        this._optionValueSources = {};
        this._storeOptionsAsProperties = false;
        this._actionHandler = null;
        this._executableHandler = false;
        this._executableFile = null;
        this._executableDir = null;
        this._defaultCommandName = null;
        this._exitCallback = null;
        this._aliases = [];
        this._combineFlagAndOptionalValue = true;
        this._description = "";
        this._summary = "";
        this._argsDescription = void 0;
        this._enablePositionalOptions = false;
        this._passThroughOptions = false;
        this._lifeCycleHooks = {};
        this._showHelpAfterError = false;
        this._showSuggestionAfterError = true;
        this._savedState = null;
        this._outputConfiguration = {
          writeOut: (str) => process3.stdout.write(str),
          writeErr: (str) => process3.stderr.write(str),
          outputError: (str, write) => write(str),
          getOutHelpWidth: () => process3.stdout.isTTY ? process3.stdout.columns : void 0,
          getErrHelpWidth: () => process3.stderr.isTTY ? process3.stderr.columns : void 0,
          getOutHasColors: () => useColor() ?? (process3.stdout.isTTY && process3.stdout.hasColors?.()),
          getErrHasColors: () => useColor() ?? (process3.stderr.isTTY && process3.stderr.hasColors?.()),
          stripColor: (str) => stripColor(str)
        };
        this._hidden = false;
        this._helpOption = void 0;
        this._addImplicitHelpCommand = void 0;
        this._helpCommand = void 0;
        this._helpConfiguration = {};
        this._helpGroupHeading = void 0;
        this._defaultCommandGroup = void 0;
        this._defaultOptionGroup = void 0;
      }
      /**
       * Copy settings that are useful to have in common across root command and subcommands.
       *
       * (Used internally when adding a command using `.command()` so subcommands inherit parent settings.)
       *
       * @param {Command} sourceCommand
       * @return {Command} `this` command for chaining
       */
      copyInheritedSettings(sourceCommand) {
        this._outputConfiguration = sourceCommand._outputConfiguration;
        this._helpOption = sourceCommand._helpOption;
        this._helpCommand = sourceCommand._helpCommand;
        this._helpConfiguration = sourceCommand._helpConfiguration;
        this._exitCallback = sourceCommand._exitCallback;
        this._storeOptionsAsProperties = sourceCommand._storeOptionsAsProperties;
        this._combineFlagAndOptionalValue = sourceCommand._combineFlagAndOptionalValue;
        this._allowExcessArguments = sourceCommand._allowExcessArguments;
        this._enablePositionalOptions = sourceCommand._enablePositionalOptions;
        this._showHelpAfterError = sourceCommand._showHelpAfterError;
        this._showSuggestionAfterError = sourceCommand._showSuggestionAfterError;
        return this;
      }
      /**
       * @returns {Command[]}
       * @private
       */
      _getCommandAndAncestors() {
        const result = [];
        for (let command = this; command; command = command.parent) {
          result.push(command);
        }
        return result;
      }
      /**
       * Define a command.
       *
       * There are two styles of command: pay attention to where to put the description.
       *
       * @example
       * // Command implemented using action handler (description is supplied separately to `.command`)
       * program
       *   .command('clone <source> [destination]')
       *   .description('clone a repository into a newly created directory')
       *   .action((source, destination) => {
       *     console.log('clone command called');
       *   });
       *
       * // Command implemented using separate executable file (description is second parameter to `.command`)
       * program
       *   .command('start <service>', 'start named service')
       *   .command('stop [service]', 'stop named service, or all if no name supplied');
       *
       * @param {string} nameAndArgs - command name and arguments, args are `<required>` or `[optional]` and last may also be `variadic...`
       * @param {(object | string)} [actionOptsOrExecDesc] - configuration options (for action), or description (for executable)
       * @param {object} [execOpts] - configuration options (for executable)
       * @return {Command} returns new command for action handler, or `this` for executable command
       */
      command(nameAndArgs, actionOptsOrExecDesc, execOpts) {
        let desc = actionOptsOrExecDesc;
        let opts = execOpts;
        if (typeof desc === "object" && desc !== null) {
          opts = desc;
          desc = null;
        }
        opts = opts || {};
        const [, name, args] = nameAndArgs.match(/([^ ]+) *(.*)/);
        const cmd = this.createCommand(name);
        if (desc) {
          cmd.description(desc);
          cmd._executableHandler = true;
        }
        if (opts.isDefault) this._defaultCommandName = cmd._name;
        cmd._hidden = !!(opts.noHelp || opts.hidden);
        cmd._executableFile = opts.executableFile || null;
        if (args) cmd.arguments(args);
        this._registerCommand(cmd);
        cmd.parent = this;
        cmd.copyInheritedSettings(this);
        if (desc) return this;
        return cmd;
      }
      /**
       * Factory routine to create a new unattached command.
       *
       * See .command() for creating an attached subcommand, which uses this routine to
       * create the command. You can override createCommand to customise subcommands.
       *
       * @param {string} [name]
       * @return {Command} new command
       */
      createCommand(name) {
        return new _Command(name);
      }
      /**
       * You can customise the help with a subclass of Help by overriding createHelp,
       * or by overriding Help properties using configureHelp().
       *
       * @return {Help}
       */
      createHelp() {
        return Object.assign(new Help2(), this.configureHelp());
      }
      /**
       * You can customise the help by overriding Help properties using configureHelp(),
       * or with a subclass of Help by overriding createHelp().
       *
       * @param {object} [configuration] - configuration options
       * @return {(Command | object)} `this` command for chaining, or stored configuration
       */
      configureHelp(configuration) {
        if (configuration === void 0) return this._helpConfiguration;
        this._helpConfiguration = configuration;
        return this;
      }
      /**
       * The default output goes to stdout and stderr. You can customise this for special
       * applications. You can also customise the display of errors by overriding outputError.
       *
       * The configuration properties are all functions:
       *
       *     // change how output being written, defaults to stdout and stderr
       *     writeOut(str)
       *     writeErr(str)
       *     // change how output being written for errors, defaults to writeErr
       *     outputError(str, write) // used for displaying errors and not used for displaying help
       *     // specify width for wrapping help
       *     getOutHelpWidth()
       *     getErrHelpWidth()
       *     // color support, currently only used with Help
       *     getOutHasColors()
       *     getErrHasColors()
       *     stripColor() // used to remove ANSI escape codes if output does not have colors
       *
       * @param {object} [configuration] - configuration options
       * @return {(Command | object)} `this` command for chaining, or stored configuration
       */
      configureOutput(configuration) {
        if (configuration === void 0) return this._outputConfiguration;
        this._outputConfiguration = {
          ...this._outputConfiguration,
          ...configuration
        };
        return this;
      }
      /**
       * Display the help or a custom message after an error occurs.
       *
       * @param {(boolean|string)} [displayHelp]
       * @return {Command} `this` command for chaining
       */
      showHelpAfterError(displayHelp = true) {
        if (typeof displayHelp !== "string") displayHelp = !!displayHelp;
        this._showHelpAfterError = displayHelp;
        return this;
      }
      /**
       * Display suggestion of similar commands for unknown commands, or options for unknown options.
       *
       * @param {boolean} [displaySuggestion]
       * @return {Command} `this` command for chaining
       */
      showSuggestionAfterError(displaySuggestion = true) {
        this._showSuggestionAfterError = !!displaySuggestion;
        return this;
      }
      /**
       * Add a prepared subcommand.
       *
       * See .command() for creating an attached subcommand which inherits settings from its parent.
       *
       * @param {Command} cmd - new subcommand
       * @param {object} [opts] - configuration options
       * @return {Command} `this` command for chaining
       */
      addCommand(cmd, opts) {
        if (!cmd._name) {
          throw new Error(`Command passed to .addCommand() must have a name
- specify the name in Command constructor or using .name()`);
        }
        opts = opts || {};
        if (opts.isDefault) this._defaultCommandName = cmd._name;
        if (opts.noHelp || opts.hidden) cmd._hidden = true;
        this._registerCommand(cmd);
        cmd.parent = this;
        cmd._checkForBrokenPassThrough();
        return this;
      }
      /**
       * Factory routine to create a new unattached argument.
       *
       * See .argument() for creating an attached argument, which uses this routine to
       * create the argument. You can override createArgument to return a custom argument.
       *
       * @param {string} name
       * @param {string} [description]
       * @return {Argument} new argument
       */
      createArgument(name, description) {
        return new Argument2(name, description);
      }
      /**
       * Define argument syntax for command.
       *
       * The default is that the argument is required, and you can explicitly
       * indicate this with <> around the name. Put [] around the name for an optional argument.
       *
       * @example
       * program.argument('<input-file>');
       * program.argument('[output-file]');
       *
       * @param {string} name
       * @param {string} [description]
       * @param {(Function|*)} [parseArg] - custom argument processing function or default value
       * @param {*} [defaultValue]
       * @return {Command} `this` command for chaining
       */
      argument(name, description, parseArg, defaultValue) {
        const argument = this.createArgument(name, description);
        if (typeof parseArg === "function") {
          argument.default(defaultValue).argParser(parseArg);
        } else {
          argument.default(parseArg);
        }
        this.addArgument(argument);
        return this;
      }
      /**
       * Define argument syntax for command, adding multiple at once (without descriptions).
       *
       * See also .argument().
       *
       * @example
       * program.arguments('<cmd> [env]');
       *
       * @param {string} names
       * @return {Command} `this` command for chaining
       */
      arguments(names) {
        names.trim().split(/ +/).forEach((detail) => {
          this.argument(detail);
        });
        return this;
      }
      /**
       * Define argument syntax for command, adding a prepared argument.
       *
       * @param {Argument} argument
       * @return {Command} `this` command for chaining
       */
      addArgument(argument) {
        const previousArgument = this.registeredArguments.slice(-1)[0];
        if (previousArgument?.variadic) {
          throw new Error(
            `only the last argument can be variadic '${previousArgument.name()}'`
          );
        }
        if (argument.required && argument.defaultValue !== void 0 && argument.parseArg === void 0) {
          throw new Error(
            `a default value for a required argument is never used: '${argument.name()}'`
          );
        }
        this.registeredArguments.push(argument);
        return this;
      }
      /**
       * Customise or override default help command. By default a help command is automatically added if your command has subcommands.
       *
       * @example
       *    program.helpCommand('help [cmd]');
       *    program.helpCommand('help [cmd]', 'show help');
       *    program.helpCommand(false); // suppress default help command
       *    program.helpCommand(true); // add help command even if no subcommands
       *
       * @param {string|boolean} enableOrNameAndArgs - enable with custom name and/or arguments, or boolean to override whether added
       * @param {string} [description] - custom description
       * @return {Command} `this` command for chaining
       */
      helpCommand(enableOrNameAndArgs, description) {
        if (typeof enableOrNameAndArgs === "boolean") {
          this._addImplicitHelpCommand = enableOrNameAndArgs;
          if (enableOrNameAndArgs && this._defaultCommandGroup) {
            this._initCommandGroup(this._getHelpCommand());
          }
          return this;
        }
        const nameAndArgs = enableOrNameAndArgs ?? "help [command]";
        const [, helpName, helpArgs] = nameAndArgs.match(/([^ ]+) *(.*)/);
        const helpDescription = description ?? "display help for command";
        const helpCommand = this.createCommand(helpName);
        helpCommand.helpOption(false);
        if (helpArgs) helpCommand.arguments(helpArgs);
        if (helpDescription) helpCommand.description(helpDescription);
        this._addImplicitHelpCommand = true;
        this._helpCommand = helpCommand;
        if (enableOrNameAndArgs || description) this._initCommandGroup(helpCommand);
        return this;
      }
      /**
       * Add prepared custom help command.
       *
       * @param {(Command|string|boolean)} helpCommand - custom help command, or deprecated enableOrNameAndArgs as for `.helpCommand()`
       * @param {string} [deprecatedDescription] - deprecated custom description used with custom name only
       * @return {Command} `this` command for chaining
       */
      addHelpCommand(helpCommand, deprecatedDescription) {
        if (typeof helpCommand !== "object") {
          this.helpCommand(helpCommand, deprecatedDescription);
          return this;
        }
        this._addImplicitHelpCommand = true;
        this._helpCommand = helpCommand;
        this._initCommandGroup(helpCommand);
        return this;
      }
      /**
       * Lazy create help command.
       *
       * @return {(Command|null)}
       * @package
       */
      _getHelpCommand() {
        const hasImplicitHelpCommand = this._addImplicitHelpCommand ?? (this.commands.length && !this._actionHandler && !this._findCommand("help"));
        if (hasImplicitHelpCommand) {
          if (this._helpCommand === void 0) {
            this.helpCommand(void 0, void 0);
          }
          return this._helpCommand;
        }
        return null;
      }
      /**
       * Add hook for life cycle event.
       *
       * @param {string} event
       * @param {Function} listener
       * @return {Command} `this` command for chaining
       */
      hook(event, listener) {
        const allowedValues = ["preSubcommand", "preAction", "postAction"];
        if (!allowedValues.includes(event)) {
          throw new Error(`Unexpected value for event passed to hook : '${event}'.
Expecting one of '${allowedValues.join("', '")}'`);
        }
        if (this._lifeCycleHooks[event]) {
          this._lifeCycleHooks[event].push(listener);
        } else {
          this._lifeCycleHooks[event] = [listener];
        }
        return this;
      }
      /**
       * Register callback to use as replacement for calling process.exit.
       *
       * @param {Function} [fn] optional callback which will be passed a CommanderError, defaults to throwing
       * @return {Command} `this` command for chaining
       */
      exitOverride(fn) {
        if (fn) {
          this._exitCallback = fn;
        } else {
          this._exitCallback = (err) => {
            if (err.code !== "commander.executeSubCommandAsync") {
              throw err;
            } else {
            }
          };
        }
        return this;
      }
      /**
       * Call process.exit, and _exitCallback if defined.
       *
       * @param {number} exitCode exit code for using with process.exit
       * @param {string} code an id string representing the error
       * @param {string} message human-readable description of the error
       * @return never
       * @private
       */
      _exit(exitCode, code, message) {
        if (this._exitCallback) {
          this._exitCallback(new CommanderError2(exitCode, code, message));
        }
        process3.exit(exitCode);
      }
      /**
       * Register callback `fn` for the command.
       *
       * @example
       * program
       *   .command('serve')
       *   .description('start service')
       *   .action(function() {
       *      // do work here
       *   });
       *
       * @param {Function} fn
       * @return {Command} `this` command for chaining
       */
      action(fn) {
        const listener = (args) => {
          const expectedArgsCount = this.registeredArguments.length;
          const actionArgs = args.slice(0, expectedArgsCount);
          if (this._storeOptionsAsProperties) {
            actionArgs[expectedArgsCount] = this;
          } else {
            actionArgs[expectedArgsCount] = this.opts();
          }
          actionArgs.push(this);
          return fn.apply(this, actionArgs);
        };
        this._actionHandler = listener;
        return this;
      }
      /**
       * Factory routine to create a new unattached option.
       *
       * See .option() for creating an attached option, which uses this routine to
       * create the option. You can override createOption to return a custom option.
       *
       * @param {string} flags
       * @param {string} [description]
       * @return {Option} new option
       */
      createOption(flags, description) {
        return new Option2(flags, description);
      }
      /**
       * Wrap parseArgs to catch 'commander.invalidArgument'.
       *
       * @param {(Option | Argument)} target
       * @param {string} value
       * @param {*} previous
       * @param {string} invalidArgumentMessage
       * @private
       */
      _callParseArg(target, value, previous, invalidArgumentMessage) {
        try {
          return target.parseArg(value, previous);
        } catch (err) {
          if (err.code === "commander.invalidArgument") {
            const message = `${invalidArgumentMessage} ${err.message}`;
            this.error(message, { exitCode: err.exitCode, code: err.code });
          }
          throw err;
        }
      }
      /**
       * Check for option flag conflicts.
       * Register option if no conflicts found, or throw on conflict.
       *
       * @param {Option} option
       * @private
       */
      _registerOption(option) {
        const matchingOption = option.short && this._findOption(option.short) || option.long && this._findOption(option.long);
        if (matchingOption) {
          const matchingFlag = option.long && this._findOption(option.long) ? option.long : option.short;
          throw new Error(`Cannot add option '${option.flags}'${this._name && ` to command '${this._name}'`} due to conflicting flag '${matchingFlag}'
-  already used by option '${matchingOption.flags}'`);
        }
        this._initOptionGroup(option);
        this.options.push(option);
      }
      /**
       * Check for command name and alias conflicts with existing commands.
       * Register command if no conflicts found, or throw on conflict.
       *
       * @param {Command} command
       * @private
       */
      _registerCommand(command) {
        const knownBy = (cmd) => {
          return [cmd.name()].concat(cmd.aliases());
        };
        const alreadyUsed = knownBy(command).find(
          (name) => this._findCommand(name)
        );
        if (alreadyUsed) {
          const existingCmd = knownBy(this._findCommand(alreadyUsed)).join("|");
          const newCmd = knownBy(command).join("|");
          throw new Error(
            `cannot add command '${newCmd}' as already have command '${existingCmd}'`
          );
        }
        this._initCommandGroup(command);
        this.commands.push(command);
      }
      /**
       * Add an option.
       *
       * @param {Option} option
       * @return {Command} `this` command for chaining
       */
      addOption(option) {
        this._registerOption(option);
        const oname = option.name();
        const name = option.attributeName();
        if (option.negate) {
          const positiveLongFlag = option.long.replace(/^--no-/, "--");
          if (!this._findOption(positiveLongFlag)) {
            this.setOptionValueWithSource(
              name,
              option.defaultValue === void 0 ? true : option.defaultValue,
              "default"
            );
          }
        } else if (option.defaultValue !== void 0) {
          this.setOptionValueWithSource(name, option.defaultValue, "default");
        }
        const handleOptionValue = (val, invalidValueMessage, valueSource) => {
          if (val == null && option.presetArg !== void 0) {
            val = option.presetArg;
          }
          const oldValue = this.getOptionValue(name);
          if (val !== null && option.parseArg) {
            val = this._callParseArg(option, val, oldValue, invalidValueMessage);
          } else if (val !== null && option.variadic) {
            val = option._collectValue(val, oldValue);
          }
          if (val == null) {
            if (option.negate) {
              val = false;
            } else if (option.isBoolean() || option.optional) {
              val = true;
            } else {
              val = "";
            }
          }
          this.setOptionValueWithSource(name, val, valueSource);
        };
        this.on("option:" + oname, (val) => {
          const invalidValueMessage = `error: option '${option.flags}' argument '${val}' is invalid.`;
          handleOptionValue(val, invalidValueMessage, "cli");
        });
        if (option.envVar) {
          this.on("optionEnv:" + oname, (val) => {
            const invalidValueMessage = `error: option '${option.flags}' value '${val}' from env '${option.envVar}' is invalid.`;
            handleOptionValue(val, invalidValueMessage, "env");
          });
        }
        return this;
      }
      /**
       * Internal implementation shared by .option() and .requiredOption()
       *
       * @return {Command} `this` command for chaining
       * @private
       */
      _optionEx(config, flags, description, fn, defaultValue) {
        if (typeof flags === "object" && flags instanceof Option2) {
          throw new Error(
            "To add an Option object use addOption() instead of option() or requiredOption()"
          );
        }
        const option = this.createOption(flags, description);
        option.makeOptionMandatory(!!config.mandatory);
        if (typeof fn === "function") {
          option.default(defaultValue).argParser(fn);
        } else if (fn instanceof RegExp) {
          const regex = fn;
          fn = (val, def) => {
            const m = regex.exec(val);
            return m ? m[0] : def;
          };
          option.default(defaultValue).argParser(fn);
        } else {
          option.default(fn);
        }
        return this.addOption(option);
      }
      /**
       * Define option with `flags`, `description`, and optional argument parsing function or `defaultValue` or both.
       *
       * The `flags` string contains the short and/or long flags, separated by comma, a pipe or space. A required
       * option-argument is indicated by `<>` and an optional option-argument by `[]`.
       *
       * See the README for more details, and see also addOption() and requiredOption().
       *
       * @example
       * program
       *     .option('-p, --pepper', 'add pepper')
       *     .option('--pt, --pizza-type <TYPE>', 'type of pizza') // required option-argument
       *     .option('-c, --cheese [CHEESE]', 'add extra cheese', 'mozzarella') // optional option-argument with default
       *     .option('-t, --tip <VALUE>', 'add tip to purchase cost', parseFloat) // custom parse function
       *
       * @param {string} flags
       * @param {string} [description]
       * @param {(Function|*)} [parseArg] - custom option processing function or default value
       * @param {*} [defaultValue]
       * @return {Command} `this` command for chaining
       */
      option(flags, description, parseArg, defaultValue) {
        return this._optionEx({}, flags, description, parseArg, defaultValue);
      }
      /**
       * Add a required option which must have a value after parsing. This usually means
       * the option must be specified on the command line. (Otherwise the same as .option().)
       *
       * The `flags` string contains the short and/or long flags, separated by comma, a pipe or space.
       *
       * @param {string} flags
       * @param {string} [description]
       * @param {(Function|*)} [parseArg] - custom option processing function or default value
       * @param {*} [defaultValue]
       * @return {Command} `this` command for chaining
       */
      requiredOption(flags, description, parseArg, defaultValue) {
        return this._optionEx(
          { mandatory: true },
          flags,
          description,
          parseArg,
          defaultValue
        );
      }
      /**
       * Alter parsing of short flags with optional values.
       *
       * @example
       * // for `.option('-f,--flag [value]'):
       * program.combineFlagAndOptionalValue(true);  // `-f80` is treated like `--flag=80`, this is the default behaviour
       * program.combineFlagAndOptionalValue(false) // `-fb` is treated like `-f -b`
       *
       * @param {boolean} [combine] - if `true` or omitted, an optional value can be specified directly after the flag.
       * @return {Command} `this` command for chaining
       */
      combineFlagAndOptionalValue(combine = true) {
        this._combineFlagAndOptionalValue = !!combine;
        return this;
      }
      /**
       * Allow unknown options on the command line.
       *
       * @param {boolean} [allowUnknown] - if `true` or omitted, no error will be thrown for unknown options.
       * @return {Command} `this` command for chaining
       */
      allowUnknownOption(allowUnknown = true) {
        this._allowUnknownOption = !!allowUnknown;
        return this;
      }
      /**
       * Allow excess command-arguments on the command line. Pass false to make excess arguments an error.
       *
       * @param {boolean} [allowExcess] - if `true` or omitted, no error will be thrown for excess arguments.
       * @return {Command} `this` command for chaining
       */
      allowExcessArguments(allowExcess = true) {
        this._allowExcessArguments = !!allowExcess;
        return this;
      }
      /**
       * Enable positional options. Positional means global options are specified before subcommands which lets
       * subcommands reuse the same option names, and also enables subcommands to turn on passThroughOptions.
       * The default behaviour is non-positional and global options may appear anywhere on the command line.
       *
       * @param {boolean} [positional]
       * @return {Command} `this` command for chaining
       */
      enablePositionalOptions(positional = true) {
        this._enablePositionalOptions = !!positional;
        return this;
      }
      /**
       * Pass through options that come after command-arguments rather than treat them as command-options,
       * so actual command-options come before command-arguments. Turning this on for a subcommand requires
       * positional options to have been enabled on the program (parent commands).
       * The default behaviour is non-positional and options may appear before or after command-arguments.
       *
       * @param {boolean} [passThrough] for unknown options.
       * @return {Command} `this` command for chaining
       */
      passThroughOptions(passThrough = true) {
        this._passThroughOptions = !!passThrough;
        this._checkForBrokenPassThrough();
        return this;
      }
      /**
       * @private
       */
      _checkForBrokenPassThrough() {
        if (this.parent && this._passThroughOptions && !this.parent._enablePositionalOptions) {
          throw new Error(
            `passThroughOptions cannot be used for '${this._name}' without turning on enablePositionalOptions for parent command(s)`
          );
        }
      }
      /**
       * Whether to store option values as properties on command object,
       * or store separately (specify false). In both cases the option values can be accessed using .opts().
       *
       * @param {boolean} [storeAsProperties=true]
       * @return {Command} `this` command for chaining
       */
      storeOptionsAsProperties(storeAsProperties = true) {
        if (this.options.length) {
          throw new Error("call .storeOptionsAsProperties() before adding options");
        }
        if (Object.keys(this._optionValues).length) {
          throw new Error(
            "call .storeOptionsAsProperties() before setting option values"
          );
        }
        this._storeOptionsAsProperties = !!storeAsProperties;
        return this;
      }
      /**
       * Retrieve option value.
       *
       * @param {string} key
       * @return {object} value
       */
      getOptionValue(key) {
        if (this._storeOptionsAsProperties) {
          return this[key];
        }
        return this._optionValues[key];
      }
      /**
       * Store option value.
       *
       * @param {string} key
       * @param {object} value
       * @return {Command} `this` command for chaining
       */
      setOptionValue(key, value) {
        return this.setOptionValueWithSource(key, value, void 0);
      }
      /**
       * Store option value and where the value came from.
       *
       * @param {string} key
       * @param {object} value
       * @param {string} source - expected values are default/config/env/cli/implied
       * @return {Command} `this` command for chaining
       */
      setOptionValueWithSource(key, value, source) {
        if (this._storeOptionsAsProperties) {
          this[key] = value;
        } else {
          this._optionValues[key] = value;
        }
        this._optionValueSources[key] = source;
        return this;
      }
      /**
       * Get source of option value.
       * Expected values are default | config | env | cli | implied
       *
       * @param {string} key
       * @return {string}
       */
      getOptionValueSource(key) {
        return this._optionValueSources[key];
      }
      /**
       * Get source of option value. See also .optsWithGlobals().
       * Expected values are default | config | env | cli | implied
       *
       * @param {string} key
       * @return {string}
       */
      getOptionValueSourceWithGlobals(key) {
        let source;
        this._getCommandAndAncestors().forEach((cmd) => {
          if (cmd.getOptionValueSource(key) !== void 0) {
            source = cmd.getOptionValueSource(key);
          }
        });
        return source;
      }
      /**
       * Get user arguments from implied or explicit arguments.
       * Side-effects: set _scriptPath if args included script. Used for default program name, and subcommand searches.
       *
       * @private
       */
      _prepareUserArgs(argv, parseOptions) {
        if (argv !== void 0 && !Array.isArray(argv)) {
          throw new Error("first parameter to parse must be array or undefined");
        }
        parseOptions = parseOptions || {};
        if (argv === void 0 && parseOptions.from === void 0) {
          if (process3.versions?.electron) {
            parseOptions.from = "electron";
          }
          const execArgv = process3.execArgv ?? [];
          if (execArgv.includes("-e") || execArgv.includes("--eval") || execArgv.includes("-p") || execArgv.includes("--print")) {
            parseOptions.from = "eval";
          }
        }
        if (argv === void 0) {
          argv = process3.argv;
        }
        this.rawArgs = argv.slice();
        let userArgs;
        switch (parseOptions.from) {
          case void 0:
          case "node":
            this._scriptPath = argv[1];
            userArgs = argv.slice(2);
            break;
          case "electron":
            if (process3.defaultApp) {
              this._scriptPath = argv[1];
              userArgs = argv.slice(2);
            } else {
              userArgs = argv.slice(1);
            }
            break;
          case "user":
            userArgs = argv.slice(0);
            break;
          case "eval":
            userArgs = argv.slice(1);
            break;
          default:
            throw new Error(
              `unexpected parse option { from: '${parseOptions.from}' }`
            );
        }
        if (!this._name && this._scriptPath)
          this.nameFromFilename(this._scriptPath);
        this._name = this._name || "program";
        return userArgs;
      }
      /**
       * Parse `argv`, setting options and invoking commands when defined.
       *
       * Use parseAsync instead of parse if any of your action handlers are async.
       *
       * Call with no parameters to parse `process.argv`. Detects Electron and special node options like `node --eval`. Easy mode!
       *
       * Or call with an array of strings to parse, and optionally where the user arguments start by specifying where the arguments are `from`:
       * - `'node'`: default, `argv[0]` is the application and `argv[1]` is the script being run, with user arguments after that
       * - `'electron'`: `argv[0]` is the application and `argv[1]` varies depending on whether the electron application is packaged
       * - `'user'`: just user arguments
       *
       * @example
       * program.parse(); // parse process.argv and auto-detect electron and special node flags
       * program.parse(process.argv); // assume argv[0] is app and argv[1] is script
       * program.parse(my-args, { from: 'user' }); // just user supplied arguments, nothing special about argv[0]
       *
       * @param {string[]} [argv] - optional, defaults to process.argv
       * @param {object} [parseOptions] - optionally specify style of options with from: node/user/electron
       * @param {string} [parseOptions.from] - where the args are from: 'node', 'user', 'electron'
       * @return {Command} `this` command for chaining
       */
      parse(argv, parseOptions) {
        this._prepareForParse();
        const userArgs = this._prepareUserArgs(argv, parseOptions);
        this._parseCommand([], userArgs);
        return this;
      }
      /**
       * Parse `argv`, setting options and invoking commands when defined.
       *
       * Call with no parameters to parse `process.argv`. Detects Electron and special node options like `node --eval`. Easy mode!
       *
       * Or call with an array of strings to parse, and optionally where the user arguments start by specifying where the arguments are `from`:
       * - `'node'`: default, `argv[0]` is the application and `argv[1]` is the script being run, with user arguments after that
       * - `'electron'`: `argv[0]` is the application and `argv[1]` varies depending on whether the electron application is packaged
       * - `'user'`: just user arguments
       *
       * @example
       * await program.parseAsync(); // parse process.argv and auto-detect electron and special node flags
       * await program.parseAsync(process.argv); // assume argv[0] is app and argv[1] is script
       * await program.parseAsync(my-args, { from: 'user' }); // just user supplied arguments, nothing special about argv[0]
       *
       * @param {string[]} [argv]
       * @param {object} [parseOptions]
       * @param {string} parseOptions.from - where the args are from: 'node', 'user', 'electron'
       * @return {Promise}
       */
      async parseAsync(argv, parseOptions) {
        this._prepareForParse();
        const userArgs = this._prepareUserArgs(argv, parseOptions);
        await this._parseCommand([], userArgs);
        return this;
      }
      _prepareForParse() {
        if (this._savedState === null) {
          this.saveStateBeforeParse();
        } else {
          this.restoreStateBeforeParse();
        }
      }
      /**
       * Called the first time parse is called to save state and allow a restore before subsequent calls to parse.
       * Not usually called directly, but available for subclasses to save their custom state.
       *
       * This is called in a lazy way. Only commands used in parsing chain will have state saved.
       */
      saveStateBeforeParse() {
        this._savedState = {
          // name is stable if supplied by author, but may be unspecified for root command and deduced during parsing
          _name: this._name,
          // option values before parse have default values (including false for negated options)
          // shallow clones
          _optionValues: { ...this._optionValues },
          _optionValueSources: { ...this._optionValueSources }
        };
      }
      /**
       * Restore state before parse for calls after the first.
       * Not usually called directly, but available for subclasses to save their custom state.
       *
       * This is called in a lazy way. Only commands used in parsing chain will have state restored.
       */
      restoreStateBeforeParse() {
        if (this._storeOptionsAsProperties)
          throw new Error(`Can not call parse again when storeOptionsAsProperties is true.
- either make a new Command for each call to parse, or stop storing options as properties`);
        this._name = this._savedState._name;
        this._scriptPath = null;
        this.rawArgs = [];
        this._optionValues = { ...this._savedState._optionValues };
        this._optionValueSources = { ...this._savedState._optionValueSources };
        this.args = [];
        this.processedArgs = [];
      }
      /**
       * Throw if expected executable is missing. Add lots of help for author.
       *
       * @param {string} executableFile
       * @param {string} executableDir
       * @param {string} subcommandName
       */
      _checkForMissingExecutable(executableFile, executableDir, subcommandName) {
        if (fs2.existsSync(executableFile)) return;
        const executableDirMessage = executableDir ? `searched for local subcommand relative to directory '${executableDir}'` : "no directory for search for local subcommand, use .executableDir() to supply a custom directory";
        const executableMissing = `'${executableFile}' does not exist
 - if '${subcommandName}' is not meant to be an executable command, remove description parameter from '.command()' and use '.description()' instead
 - if the default executable name is not suitable, use the executableFile option to supply a custom name or path
 - ${executableDirMessage}`;
        throw new Error(executableMissing);
      }
      /**
       * Execute a sub-command executable.
       *
       * @private
       */
      _executeSubCommand(subcommand, args) {
        args = args.slice();
        let launchWithNode = false;
        const sourceExt = [".js", ".ts", ".tsx", ".mjs", ".cjs"];
        function findFile(baseDir, baseName) {
          const localBin = path2.resolve(baseDir, baseName);
          if (fs2.existsSync(localBin)) return localBin;
          if (sourceExt.includes(path2.extname(baseName))) return void 0;
          const foundExt = sourceExt.find(
            (ext) => fs2.existsSync(`${localBin}${ext}`)
          );
          if (foundExt) return `${localBin}${foundExt}`;
          return void 0;
        }
        this._checkForMissingMandatoryOptions();
        this._checkForConflictingOptions();
        let executableFile = subcommand._executableFile || `${this._name}-${subcommand._name}`;
        let executableDir = this._executableDir || "";
        if (this._scriptPath) {
          let resolvedScriptPath;
          try {
            resolvedScriptPath = fs2.realpathSync(this._scriptPath);
          } catch {
            resolvedScriptPath = this._scriptPath;
          }
          executableDir = path2.resolve(
            path2.dirname(resolvedScriptPath),
            executableDir
          );
        }
        if (executableDir) {
          let localFile = findFile(executableDir, executableFile);
          if (!localFile && !subcommand._executableFile && this._scriptPath) {
            const legacyName = path2.basename(
              this._scriptPath,
              path2.extname(this._scriptPath)
            );
            if (legacyName !== this._name) {
              localFile = findFile(
                executableDir,
                `${legacyName}-${subcommand._name}`
              );
            }
          }
          executableFile = localFile || executableFile;
        }
        launchWithNode = sourceExt.includes(path2.extname(executableFile));
        let proc;
        if (process3.platform !== "win32") {
          if (launchWithNode) {
            args.unshift(executableFile);
            args = incrementNodeInspectorPort(process3.execArgv).concat(args);
            proc = childProcess.spawn(process3.argv[0], args, { stdio: "inherit" });
          } else {
            proc = childProcess.spawn(executableFile, args, { stdio: "inherit" });
          }
        } else {
          this._checkForMissingExecutable(
            executableFile,
            executableDir,
            subcommand._name
          );
          args.unshift(executableFile);
          args = incrementNodeInspectorPort(process3.execArgv).concat(args);
          proc = childProcess.spawn(process3.execPath, args, { stdio: "inherit" });
        }
        if (!proc.killed) {
          const signals = ["SIGUSR1", "SIGUSR2", "SIGTERM", "SIGINT", "SIGHUP"];
          signals.forEach((signal) => {
            process3.on(signal, () => {
              if (proc.killed === false && proc.exitCode === null) {
                proc.kill(signal);
              }
            });
          });
        }
        const exitCallback = this._exitCallback;
        proc.on("close", (code) => {
          code = code ?? 1;
          if (!exitCallback) {
            process3.exit(code);
          } else {
            exitCallback(
              new CommanderError2(
                code,
                "commander.executeSubCommandAsync",
                "(close)"
              )
            );
          }
        });
        proc.on("error", (err) => {
          if (err.code === "ENOENT") {
            this._checkForMissingExecutable(
              executableFile,
              executableDir,
              subcommand._name
            );
          } else if (err.code === "EACCES") {
            throw new Error(`'${executableFile}' not executable`);
          }
          if (!exitCallback) {
            process3.exit(1);
          } else {
            const wrappedError = new CommanderError2(
              1,
              "commander.executeSubCommandAsync",
              "(error)"
            );
            wrappedError.nestedError = err;
            exitCallback(wrappedError);
          }
        });
        this.runningCommand = proc;
      }
      /**
       * @private
       */
      _dispatchSubcommand(commandName, operands, unknown) {
        const subCommand = this._findCommand(commandName);
        if (!subCommand) this.help({ error: true });
        subCommand._prepareForParse();
        let promiseChain;
        promiseChain = this._chainOrCallSubCommandHook(
          promiseChain,
          subCommand,
          "preSubcommand"
        );
        promiseChain = this._chainOrCall(promiseChain, () => {
          if (subCommand._executableHandler) {
            this._executeSubCommand(subCommand, operands.concat(unknown));
          } else {
            return subCommand._parseCommand(operands, unknown);
          }
        });
        return promiseChain;
      }
      /**
       * Invoke help directly if possible, or dispatch if necessary.
       * e.g. help foo
       *
       * @private
       */
      _dispatchHelpCommand(subcommandName) {
        if (!subcommandName) {
          this.help();
        }
        const subCommand = this._findCommand(subcommandName);
        if (subCommand && !subCommand._executableHandler) {
          subCommand.help();
        }
        return this._dispatchSubcommand(
          subcommandName,
          [],
          [this._getHelpOption()?.long ?? this._getHelpOption()?.short ?? "--help"]
        );
      }
      /**
       * Check this.args against expected this.registeredArguments.
       *
       * @private
       */
      _checkNumberOfArguments() {
        this.registeredArguments.forEach((arg, i2) => {
          if (arg.required && this.args[i2] == null) {
            this.missingArgument(arg.name());
          }
        });
        if (this.registeredArguments.length > 0 && this.registeredArguments[this.registeredArguments.length - 1].variadic) {
          return;
        }
        if (this.args.length > this.registeredArguments.length) {
          this._excessArguments(this.args);
        }
      }
      /**
       * Process this.args using this.registeredArguments and save as this.processedArgs!
       *
       * @private
       */
      _processArguments() {
        const myParseArg = (argument, value, previous) => {
          let parsedValue = value;
          if (value !== null && argument.parseArg) {
            const invalidValueMessage = `error: command-argument value '${value}' is invalid for argument '${argument.name()}'.`;
            parsedValue = this._callParseArg(
              argument,
              value,
              previous,
              invalidValueMessage
            );
          }
          return parsedValue;
        };
        this._checkNumberOfArguments();
        const processedArgs = [];
        this.registeredArguments.forEach((declaredArg, index) => {
          let value = declaredArg.defaultValue;
          if (declaredArg.variadic) {
            if (index < this.args.length) {
              value = this.args.slice(index);
              if (declaredArg.parseArg) {
                value = value.reduce((processed, v) => {
                  return myParseArg(declaredArg, v, processed);
                }, declaredArg.defaultValue);
              }
            } else if (value === void 0) {
              value = [];
            }
          } else if (index < this.args.length) {
            value = this.args[index];
            if (declaredArg.parseArg) {
              value = myParseArg(declaredArg, value, declaredArg.defaultValue);
            }
          }
          processedArgs[index] = value;
        });
        this.processedArgs = processedArgs;
      }
      /**
       * Once we have a promise we chain, but call synchronously until then.
       *
       * @param {(Promise|undefined)} promise
       * @param {Function} fn
       * @return {(Promise|undefined)}
       * @private
       */
      _chainOrCall(promise, fn) {
        if (promise?.then && typeof promise.then === "function") {
          return promise.then(() => fn());
        }
        return fn();
      }
      /**
       *
       * @param {(Promise|undefined)} promise
       * @param {string} event
       * @return {(Promise|undefined)}
       * @private
       */
      _chainOrCallHooks(promise, event) {
        let result = promise;
        const hooks = [];
        this._getCommandAndAncestors().reverse().filter((cmd) => cmd._lifeCycleHooks[event] !== void 0).forEach((hookedCommand) => {
          hookedCommand._lifeCycleHooks[event].forEach((callback) => {
            hooks.push({ hookedCommand, callback });
          });
        });
        if (event === "postAction") {
          hooks.reverse();
        }
        hooks.forEach((hookDetail) => {
          result = this._chainOrCall(result, () => {
            return hookDetail.callback(hookDetail.hookedCommand, this);
          });
        });
        return result;
      }
      /**
       *
       * @param {(Promise|undefined)} promise
       * @param {Command} subCommand
       * @param {string} event
       * @return {(Promise|undefined)}
       * @private
       */
      _chainOrCallSubCommandHook(promise, subCommand, event) {
        let result = promise;
        if (this._lifeCycleHooks[event] !== void 0) {
          this._lifeCycleHooks[event].forEach((hook) => {
            result = this._chainOrCall(result, () => {
              return hook(this, subCommand);
            });
          });
        }
        return result;
      }
      /**
       * Process arguments in context of this command.
       * Returns action result, in case it is a promise.
       *
       * @private
       */
      _parseCommand(operands, unknown) {
        const parsed = this.parseOptions(unknown);
        this._parseOptionsEnv();
        this._parseOptionsImplied();
        operands = operands.concat(parsed.operands);
        unknown = parsed.unknown;
        this.args = operands.concat(unknown);
        if (operands && this._findCommand(operands[0])) {
          return this._dispatchSubcommand(operands[0], operands.slice(1), unknown);
        }
        if (this._getHelpCommand() && operands[0] === this._getHelpCommand().name()) {
          return this._dispatchHelpCommand(operands[1]);
        }
        if (this._defaultCommandName) {
          this._outputHelpIfRequested(unknown);
          return this._dispatchSubcommand(
            this._defaultCommandName,
            operands,
            unknown
          );
        }
        if (this.commands.length && this.args.length === 0 && !this._actionHandler && !this._defaultCommandName) {
          this.help({ error: true });
        }
        this._outputHelpIfRequested(parsed.unknown);
        this._checkForMissingMandatoryOptions();
        this._checkForConflictingOptions();
        const checkForUnknownOptions = () => {
          if (parsed.unknown.length > 0) {
            this.unknownOption(parsed.unknown[0]);
          }
        };
        const commandEvent = `command:${this.name()}`;
        if (this._actionHandler) {
          checkForUnknownOptions();
          this._processArguments();
          let promiseChain;
          promiseChain = this._chainOrCallHooks(promiseChain, "preAction");
          promiseChain = this._chainOrCall(
            promiseChain,
            () => this._actionHandler(this.processedArgs)
          );
          if (this.parent) {
            promiseChain = this._chainOrCall(promiseChain, () => {
              this.parent.emit(commandEvent, operands, unknown);
            });
          }
          promiseChain = this._chainOrCallHooks(promiseChain, "postAction");
          return promiseChain;
        }
        if (this.parent?.listenerCount(commandEvent)) {
          checkForUnknownOptions();
          this._processArguments();
          this.parent.emit(commandEvent, operands, unknown);
        } else if (operands.length) {
          if (this._findCommand("*")) {
            return this._dispatchSubcommand("*", operands, unknown);
          }
          if (this.listenerCount("command:*")) {
            this.emit("command:*", operands, unknown);
          } else if (this.commands.length) {
            this.unknownCommand();
          } else {
            checkForUnknownOptions();
            this._processArguments();
          }
        } else if (this.commands.length) {
          checkForUnknownOptions();
          this.help({ error: true });
        } else {
          checkForUnknownOptions();
          this._processArguments();
        }
      }
      /**
       * Find matching command.
       *
       * @private
       * @return {Command | undefined}
       */
      _findCommand(name) {
        if (!name) return void 0;
        return this.commands.find(
          (cmd) => cmd._name === name || cmd._aliases.includes(name)
        );
      }
      /**
       * Return an option matching `arg` if any.
       *
       * @param {string} arg
       * @return {Option}
       * @package
       */
      _findOption(arg) {
        return this.options.find((option) => option.is(arg));
      }
      /**
       * Display an error message if a mandatory option does not have a value.
       * Called after checking for help flags in leaf subcommand.
       *
       * @private
       */
      _checkForMissingMandatoryOptions() {
        this._getCommandAndAncestors().forEach((cmd) => {
          cmd.options.forEach((anOption) => {
            if (anOption.mandatory && cmd.getOptionValue(anOption.attributeName()) === void 0) {
              cmd.missingMandatoryOptionValue(anOption);
            }
          });
        });
      }
      /**
       * Display an error message if conflicting options are used together in this.
       *
       * @private
       */
      _checkForConflictingLocalOptions() {
        const definedNonDefaultOptions = this.options.filter((option) => {
          const optionKey = option.attributeName();
          if (this.getOptionValue(optionKey) === void 0) {
            return false;
          }
          return this.getOptionValueSource(optionKey) !== "default";
        });
        const optionsWithConflicting = definedNonDefaultOptions.filter(
          (option) => option.conflictsWith.length > 0
        );
        optionsWithConflicting.forEach((option) => {
          const conflictingAndDefined = definedNonDefaultOptions.find(
            (defined) => option.conflictsWith.includes(defined.attributeName())
          );
          if (conflictingAndDefined) {
            this._conflictingOption(option, conflictingAndDefined);
          }
        });
      }
      /**
       * Display an error message if conflicting options are used together.
       * Called after checking for help flags in leaf subcommand.
       *
       * @private
       */
      _checkForConflictingOptions() {
        this._getCommandAndAncestors().forEach((cmd) => {
          cmd._checkForConflictingLocalOptions();
        });
      }
      /**
       * Parse options from `argv` removing known options,
       * and return argv split into operands and unknown arguments.
       *
       * Side effects: modifies command by storing options. Does not reset state if called again.
       *
       * Examples:
       *
       *     argv => operands, unknown
       *     --known kkk op => [op], []
       *     op --known kkk => [op], []
       *     sub --unknown uuu op => [sub], [--unknown uuu op]
       *     sub -- --unknown uuu op => [sub --unknown uuu op], []
       *
       * @param {string[]} args
       * @return {{operands: string[], unknown: string[]}}
       */
      parseOptions(args) {
        const operands = [];
        const unknown = [];
        let dest = operands;
        function maybeOption(arg) {
          return arg.length > 1 && arg[0] === "-";
        }
        const negativeNumberArg = (arg) => {
          if (!/^-(\d+|\d*\.\d+)(e[+-]?\d+)?$/.test(arg)) return false;
          return !this._getCommandAndAncestors().some(
            (cmd) => cmd.options.map((opt) => opt.short).some((short) => /^-\d$/.test(short))
          );
        };
        let activeVariadicOption = null;
        let activeGroup = null;
        let i2 = 0;
        while (i2 < args.length || activeGroup) {
          const arg = activeGroup ?? args[i2++];
          activeGroup = null;
          if (arg === "--") {
            if (dest === unknown) dest.push(arg);
            dest.push(...args.slice(i2));
            break;
          }
          if (activeVariadicOption && (!maybeOption(arg) || negativeNumberArg(arg))) {
            this.emit(`option:${activeVariadicOption.name()}`, arg);
            continue;
          }
          activeVariadicOption = null;
          if (maybeOption(arg)) {
            const option = this._findOption(arg);
            if (option) {
              if (option.required) {
                const value = args[i2++];
                if (value === void 0) this.optionMissingArgument(option);
                this.emit(`option:${option.name()}`, value);
              } else if (option.optional) {
                let value = null;
                if (i2 < args.length && (!maybeOption(args[i2]) || negativeNumberArg(args[i2]))) {
                  value = args[i2++];
                }
                this.emit(`option:${option.name()}`, value);
              } else {
                this.emit(`option:${option.name()}`);
              }
              activeVariadicOption = option.variadic ? option : null;
              continue;
            }
          }
          if (arg.length > 2 && arg[0] === "-" && arg[1] !== "-") {
            const option = this._findOption(`-${arg[1]}`);
            if (option) {
              if (option.required || option.optional && this._combineFlagAndOptionalValue) {
                this.emit(`option:${option.name()}`, arg.slice(2));
              } else {
                this.emit(`option:${option.name()}`);
                activeGroup = `-${arg.slice(2)}`;
              }
              continue;
            }
          }
          if (/^--[^=]+=/.test(arg)) {
            const index = arg.indexOf("=");
            const option = this._findOption(arg.slice(0, index));
            if (option && (option.required || option.optional)) {
              this.emit(`option:${option.name()}`, arg.slice(index + 1));
              continue;
            }
          }
          if (dest === operands && maybeOption(arg) && !(this.commands.length === 0 && negativeNumberArg(arg))) {
            dest = unknown;
          }
          if ((this._enablePositionalOptions || this._passThroughOptions) && operands.length === 0 && unknown.length === 0) {
            if (this._findCommand(arg)) {
              operands.push(arg);
              unknown.push(...args.slice(i2));
              break;
            } else if (this._getHelpCommand() && arg === this._getHelpCommand().name()) {
              operands.push(arg, ...args.slice(i2));
              break;
            } else if (this._defaultCommandName) {
              unknown.push(arg, ...args.slice(i2));
              break;
            }
          }
          if (this._passThroughOptions) {
            dest.push(arg, ...args.slice(i2));
            break;
          }
          dest.push(arg);
        }
        return { operands, unknown };
      }
      /**
       * Return an object containing local option values as key-value pairs.
       *
       * @return {object}
       */
      opts() {
        if (this._storeOptionsAsProperties) {
          const result = {};
          const len = this.options.length;
          for (let i2 = 0; i2 < len; i2++) {
            const key = this.options[i2].attributeName();
            result[key] = key === this._versionOptionName ? this._version : this[key];
          }
          return result;
        }
        return this._optionValues;
      }
      /**
       * Return an object containing merged local and global option values as key-value pairs.
       *
       * @return {object}
       */
      optsWithGlobals() {
        return this._getCommandAndAncestors().reduce(
          (combinedOptions, cmd) => Object.assign(combinedOptions, cmd.opts()),
          {}
        );
      }
      /**
       * Display error message and exit (or call exitOverride).
       *
       * @param {string} message
       * @param {object} [errorOptions]
       * @param {string} [errorOptions.code] - an id string representing the error
       * @param {number} [errorOptions.exitCode] - used with process.exit
       */
      error(message, errorOptions) {
        this._outputConfiguration.outputError(
          `${message}
`,
          this._outputConfiguration.writeErr
        );
        if (typeof this._showHelpAfterError === "string") {
          this._outputConfiguration.writeErr(`${this._showHelpAfterError}
`);
        } else if (this._showHelpAfterError) {
          this._outputConfiguration.writeErr("\n");
          this.outputHelp({ error: true });
        }
        const config = errorOptions || {};
        const exitCode = config.exitCode || 1;
        const code = config.code || "commander.error";
        this._exit(exitCode, code, message);
      }
      /**
       * Apply any option related environment variables, if option does
       * not have a value from cli or client code.
       *
       * @private
       */
      _parseOptionsEnv() {
        this.options.forEach((option) => {
          if (option.envVar && option.envVar in process3.env) {
            const optionKey = option.attributeName();
            if (this.getOptionValue(optionKey) === void 0 || ["default", "config", "env"].includes(
              this.getOptionValueSource(optionKey)
            )) {
              if (option.required || option.optional) {
                this.emit(`optionEnv:${option.name()}`, process3.env[option.envVar]);
              } else {
                this.emit(`optionEnv:${option.name()}`);
              }
            }
          }
        });
      }
      /**
       * Apply any implied option values, if option is undefined or default value.
       *
       * @private
       */
      _parseOptionsImplied() {
        const dualHelper = new DualOptions(this.options);
        const hasCustomOptionValue = (optionKey) => {
          return this.getOptionValue(optionKey) !== void 0 && !["default", "implied"].includes(this.getOptionValueSource(optionKey));
        };
        this.options.filter(
          (option) => option.implied !== void 0 && hasCustomOptionValue(option.attributeName()) && dualHelper.valueFromOption(
            this.getOptionValue(option.attributeName()),
            option
          )
        ).forEach((option) => {
          Object.keys(option.implied).filter((impliedKey) => !hasCustomOptionValue(impliedKey)).forEach((impliedKey) => {
            this.setOptionValueWithSource(
              impliedKey,
              option.implied[impliedKey],
              "implied"
            );
          });
        });
      }
      /**
       * Argument `name` is missing.
       *
       * @param {string} name
       * @private
       */
      missingArgument(name) {
        const message = `error: missing required argument '${name}'`;
        this.error(message, { code: "commander.missingArgument" });
      }
      /**
       * `Option` is missing an argument.
       *
       * @param {Option} option
       * @private
       */
      optionMissingArgument(option) {
        const message = `error: option '${option.flags}' argument missing`;
        this.error(message, { code: "commander.optionMissingArgument" });
      }
      /**
       * `Option` does not have a value, and is a mandatory option.
       *
       * @param {Option} option
       * @private
       */
      missingMandatoryOptionValue(option) {
        const message = `error: required option '${option.flags}' not specified`;
        this.error(message, { code: "commander.missingMandatoryOptionValue" });
      }
      /**
       * `Option` conflicts with another option.
       *
       * @param {Option} option
       * @param {Option} conflictingOption
       * @private
       */
      _conflictingOption(option, conflictingOption) {
        const findBestOptionFromValue = (option2) => {
          const optionKey = option2.attributeName();
          const optionValue = this.getOptionValue(optionKey);
          const negativeOption = this.options.find(
            (target) => target.negate && optionKey === target.attributeName()
          );
          const positiveOption = this.options.find(
            (target) => !target.negate && optionKey === target.attributeName()
          );
          if (negativeOption && (negativeOption.presetArg === void 0 && optionValue === false || negativeOption.presetArg !== void 0 && optionValue === negativeOption.presetArg)) {
            return negativeOption;
          }
          return positiveOption || option2;
        };
        const getErrorMessage2 = (option2) => {
          const bestOption = findBestOptionFromValue(option2);
          const optionKey = bestOption.attributeName();
          const source = this.getOptionValueSource(optionKey);
          if (source === "env") {
            return `environment variable '${bestOption.envVar}'`;
          }
          return `option '${bestOption.flags}'`;
        };
        const message = `error: ${getErrorMessage2(option)} cannot be used with ${getErrorMessage2(conflictingOption)}`;
        this.error(message, { code: "commander.conflictingOption" });
      }
      /**
       * Unknown option `flag`.
       *
       * @param {string} flag
       * @private
       */
      unknownOption(flag) {
        if (this._allowUnknownOption) return;
        let suggestion = "";
        if (flag.startsWith("--") && this._showSuggestionAfterError) {
          let candidateFlags = [];
          let command = this;
          do {
            const moreFlags = command.createHelp().visibleOptions(command).filter((option) => option.long).map((option) => option.long);
            candidateFlags = candidateFlags.concat(moreFlags);
            command = command.parent;
          } while (command && !command._enablePositionalOptions);
          suggestion = suggestSimilar(flag, candidateFlags);
        }
        const message = `error: unknown option '${flag}'${suggestion}`;
        this.error(message, { code: "commander.unknownOption" });
      }
      /**
       * Excess arguments, more than expected.
       *
       * @param {string[]} receivedArgs
       * @private
       */
      _excessArguments(receivedArgs) {
        if (this._allowExcessArguments) return;
        const expected = this.registeredArguments.length;
        const s = expected === 1 ? "" : "s";
        const forSubcommand = this.parent ? ` for '${this.name()}'` : "";
        const message = `error: too many arguments${forSubcommand}. Expected ${expected} argument${s} but got ${receivedArgs.length}.`;
        this.error(message, { code: "commander.excessArguments" });
      }
      /**
       * Unknown command.
       *
       * @private
       */
      unknownCommand() {
        const unknownName = this.args[0];
        let suggestion = "";
        if (this._showSuggestionAfterError) {
          const candidateNames = [];
          this.createHelp().visibleCommands(this).forEach((command) => {
            candidateNames.push(command.name());
            if (command.alias()) candidateNames.push(command.alias());
          });
          suggestion = suggestSimilar(unknownName, candidateNames);
        }
        const message = `error: unknown command '${unknownName}'${suggestion}`;
        this.error(message, { code: "commander.unknownCommand" });
      }
      /**
       * Get or set the program version.
       *
       * This method auto-registers the "-V, --version" option which will print the version number.
       *
       * You can optionally supply the flags and description to override the defaults.
       *
       * @param {string} [str]
       * @param {string} [flags]
       * @param {string} [description]
       * @return {(this | string | undefined)} `this` command for chaining, or version string if no arguments
       */
      version(str, flags, description) {
        if (str === void 0) return this._version;
        this._version = str;
        flags = flags || "-V, --version";
        description = description || "output the version number";
        const versionOption = this.createOption(flags, description);
        this._versionOptionName = versionOption.attributeName();
        this._registerOption(versionOption);
        this.on("option:" + versionOption.name(), () => {
          this._outputConfiguration.writeOut(`${str}
`);
          this._exit(0, "commander.version", str);
        });
        return this;
      }
      /**
       * Set the description.
       *
       * @param {string} [str]
       * @param {object} [argsDescription]
       * @return {(string|Command)}
       */
      description(str, argsDescription) {
        if (str === void 0 && argsDescription === void 0)
          return this._description;
        this._description = str;
        if (argsDescription) {
          this._argsDescription = argsDescription;
        }
        return this;
      }
      /**
       * Set the summary. Used when listed as subcommand of parent.
       *
       * @param {string} [str]
       * @return {(string|Command)}
       */
      summary(str) {
        if (str === void 0) return this._summary;
        this._summary = str;
        return this;
      }
      /**
       * Set an alias for the command.
       *
       * You may call more than once to add multiple aliases. Only the first alias is shown in the auto-generated help.
       *
       * @param {string} [alias]
       * @return {(string|Command)}
       */
      alias(alias) {
        if (alias === void 0) return this._aliases[0];
        let command = this;
        if (this.commands.length !== 0 && this.commands[this.commands.length - 1]._executableHandler) {
          command = this.commands[this.commands.length - 1];
        }
        if (alias === command._name)
          throw new Error("Command alias can't be the same as its name");
        const matchingCommand = this.parent?._findCommand(alias);
        if (matchingCommand) {
          const existingCmd = [matchingCommand.name()].concat(matchingCommand.aliases()).join("|");
          throw new Error(
            `cannot add alias '${alias}' to command '${this.name()}' as already have command '${existingCmd}'`
          );
        }
        command._aliases.push(alias);
        return this;
      }
      /**
       * Set aliases for the command.
       *
       * Only the first alias is shown in the auto-generated help.
       *
       * @param {string[]} [aliases]
       * @return {(string[]|Command)}
       */
      aliases(aliases) {
        if (aliases === void 0) return this._aliases;
        aliases.forEach((alias) => this.alias(alias));
        return this;
      }
      /**
       * Set / get the command usage `str`.
       *
       * @param {string} [str]
       * @return {(string|Command)}
       */
      usage(str) {
        if (str === void 0) {
          if (this._usage) return this._usage;
          const args = this.registeredArguments.map((arg) => {
            return humanReadableArgName(arg);
          });
          return [].concat(
            this.options.length || this._helpOption !== null ? "[options]" : [],
            this.commands.length ? "[command]" : [],
            this.registeredArguments.length ? args : []
          ).join(" ");
        }
        this._usage = str;
        return this;
      }
      /**
       * Get or set the name of the command.
       *
       * @param {string} [str]
       * @return {(string|Command)}
       */
      name(str) {
        if (str === void 0) return this._name;
        this._name = str;
        return this;
      }
      /**
       * Set/get the help group heading for this subcommand in parent command's help.
       *
       * @param {string} [heading]
       * @return {Command | string}
       */
      helpGroup(heading) {
        if (heading === void 0) return this._helpGroupHeading ?? "";
        this._helpGroupHeading = heading;
        return this;
      }
      /**
       * Set/get the default help group heading for subcommands added to this command.
       * (This does not override a group set directly on the subcommand using .helpGroup().)
       *
       * @example
       * program.commandsGroup('Development Commands:);
       * program.command('watch')...
       * program.command('lint')...
       * ...
       *
       * @param {string} [heading]
       * @returns {Command | string}
       */
      commandsGroup(heading) {
        if (heading === void 0) return this._defaultCommandGroup ?? "";
        this._defaultCommandGroup = heading;
        return this;
      }
      /**
       * Set/get the default help group heading for options added to this command.
       * (This does not override a group set directly on the option using .helpGroup().)
       *
       * @example
       * program
       *   .optionsGroup('Development Options:')
       *   .option('-d, --debug', 'output extra debugging')
       *   .option('-p, --profile', 'output profiling information')
       *
       * @param {string} [heading]
       * @returns {Command | string}
       */
      optionsGroup(heading) {
        if (heading === void 0) return this._defaultOptionGroup ?? "";
        this._defaultOptionGroup = heading;
        return this;
      }
      /**
       * @param {Option} option
       * @private
       */
      _initOptionGroup(option) {
        if (this._defaultOptionGroup && !option.helpGroupHeading)
          option.helpGroup(this._defaultOptionGroup);
      }
      /**
       * @param {Command} cmd
       * @private
       */
      _initCommandGroup(cmd) {
        if (this._defaultCommandGroup && !cmd.helpGroup())
          cmd.helpGroup(this._defaultCommandGroup);
      }
      /**
       * Set the name of the command from script filename, such as process.argv[1],
       * or require.main.filename, or __filename.
       *
       * (Used internally and public although not documented in README.)
       *
       * @example
       * program.nameFromFilename(require.main.filename);
       *
       * @param {string} filename
       * @return {Command}
       */
      nameFromFilename(filename) {
        this._name = path2.basename(filename, path2.extname(filename));
        return this;
      }
      /**
       * Get or set the directory for searching for executable subcommands of this command.
       *
       * @example
       * program.executableDir(__dirname);
       * // or
       * program.executableDir('subcommands');
       *
       * @param {string} [path]
       * @return {(string|null|Command)}
       */
      executableDir(path3) {
        if (path3 === void 0) return this._executableDir;
        this._executableDir = path3;
        return this;
      }
      /**
       * Return program help documentation.
       *
       * @param {{ error: boolean }} [contextOptions] - pass {error:true} to wrap for stderr instead of stdout
       * @return {string}
       */
      helpInformation(contextOptions) {
        const helper = this.createHelp();
        const context = this._getOutputContext(contextOptions);
        helper.prepareContext({
          error: context.error,
          helpWidth: context.helpWidth,
          outputHasColors: context.hasColors
        });
        const text = helper.formatHelp(this, helper);
        if (context.hasColors) return text;
        return this._outputConfiguration.stripColor(text);
      }
      /**
       * @typedef HelpContext
       * @type {object}
       * @property {boolean} error
       * @property {number} helpWidth
       * @property {boolean} hasColors
       * @property {function} write - includes stripColor if needed
       *
       * @returns {HelpContext}
       * @private
       */
      _getOutputContext(contextOptions) {
        contextOptions = contextOptions || {};
        const error = !!contextOptions.error;
        let baseWrite;
        let hasColors;
        let helpWidth;
        if (error) {
          baseWrite = (str) => this._outputConfiguration.writeErr(str);
          hasColors = this._outputConfiguration.getErrHasColors();
          helpWidth = this._outputConfiguration.getErrHelpWidth();
        } else {
          baseWrite = (str) => this._outputConfiguration.writeOut(str);
          hasColors = this._outputConfiguration.getOutHasColors();
          helpWidth = this._outputConfiguration.getOutHelpWidth();
        }
        const write = (str) => {
          if (!hasColors) str = this._outputConfiguration.stripColor(str);
          return baseWrite(str);
        };
        return { error, write, hasColors, helpWidth };
      }
      /**
       * Output help information for this command.
       *
       * Outputs built-in help, and custom text added using `.addHelpText()`.
       *
       * @param {{ error: boolean } | Function} [contextOptions] - pass {error:true} to write to stderr instead of stdout
       */
      outputHelp(contextOptions) {
        let deprecatedCallback;
        if (typeof contextOptions === "function") {
          deprecatedCallback = contextOptions;
          contextOptions = void 0;
        }
        const outputContext = this._getOutputContext(contextOptions);
        const eventContext = {
          error: outputContext.error,
          write: outputContext.write,
          command: this
        };
        this._getCommandAndAncestors().reverse().forEach((command) => command.emit("beforeAllHelp", eventContext));
        this.emit("beforeHelp", eventContext);
        let helpInformation = this.helpInformation({ error: outputContext.error });
        if (deprecatedCallback) {
          helpInformation = deprecatedCallback(helpInformation);
          if (typeof helpInformation !== "string" && !Buffer.isBuffer(helpInformation)) {
            throw new Error("outputHelp callback must return a string or a Buffer");
          }
        }
        outputContext.write(helpInformation);
        if (this._getHelpOption()?.long) {
          this.emit(this._getHelpOption().long);
        }
        this.emit("afterHelp", eventContext);
        this._getCommandAndAncestors().forEach(
          (command) => command.emit("afterAllHelp", eventContext)
        );
      }
      /**
       * You can pass in flags and a description to customise the built-in help option.
       * Pass in false to disable the built-in help option.
       *
       * @example
       * program.helpOption('-?, --help' 'show help'); // customise
       * program.helpOption(false); // disable
       *
       * @param {(string | boolean)} flags
       * @param {string} [description]
       * @return {Command} `this` command for chaining
       */
      helpOption(flags, description) {
        if (typeof flags === "boolean") {
          if (flags) {
            if (this._helpOption === null) this._helpOption = void 0;
            if (this._defaultOptionGroup) {
              this._initOptionGroup(this._getHelpOption());
            }
          } else {
            this._helpOption = null;
          }
          return this;
        }
        this._helpOption = this.createOption(
          flags ?? "-h, --help",
          description ?? "display help for command"
        );
        if (flags || description) this._initOptionGroup(this._helpOption);
        return this;
      }
      /**
       * Lazy create help option.
       * Returns null if has been disabled with .helpOption(false).
       *
       * @returns {(Option | null)} the help option
       * @package
       */
      _getHelpOption() {
        if (this._helpOption === void 0) {
          this.helpOption(void 0, void 0);
        }
        return this._helpOption;
      }
      /**
       * Supply your own option to use for the built-in help option.
       * This is an alternative to using helpOption() to customise the flags and description etc.
       *
       * @param {Option} option
       * @return {Command} `this` command for chaining
       */
      addHelpOption(option) {
        this._helpOption = option;
        this._initOptionGroup(option);
        return this;
      }
      /**
       * Output help information and exit.
       *
       * Outputs built-in help, and custom text added using `.addHelpText()`.
       *
       * @param {{ error: boolean }} [contextOptions] - pass {error:true} to write to stderr instead of stdout
       */
      help(contextOptions) {
        this.outputHelp(contextOptions);
        let exitCode = Number(process3.exitCode ?? 0);
        if (exitCode === 0 && contextOptions && typeof contextOptions !== "function" && contextOptions.error) {
          exitCode = 1;
        }
        this._exit(exitCode, "commander.help", "(outputHelp)");
      }
      /**
       * // Do a little typing to coordinate emit and listener for the help text events.
       * @typedef HelpTextEventContext
       * @type {object}
       * @property {boolean} error
       * @property {Command} command
       * @property {function} write
       */
      /**
       * Add additional text to be displayed with the built-in help.
       *
       * Position is 'before' or 'after' to affect just this command,
       * and 'beforeAll' or 'afterAll' to affect this command and all its subcommands.
       *
       * @param {string} position - before or after built-in help
       * @param {(string | Function)} text - string to add, or a function returning a string
       * @return {Command} `this` command for chaining
       */
      addHelpText(position, text) {
        const allowedValues = ["beforeAll", "before", "after", "afterAll"];
        if (!allowedValues.includes(position)) {
          throw new Error(`Unexpected value for position to addHelpText.
Expecting one of '${allowedValues.join("', '")}'`);
        }
        const helpEvent = `${position}Help`;
        this.on(helpEvent, (context) => {
          let helpStr;
          if (typeof text === "function") {
            helpStr = text({ error: context.error, command: context.command });
          } else {
            helpStr = text;
          }
          if (helpStr) {
            context.write(`${helpStr}
`);
          }
        });
        return this;
      }
      /**
       * Output help information if help flags specified
       *
       * @param {Array} args - array of options to search for help flags
       * @private
       */
      _outputHelpIfRequested(args) {
        const helpOption = this._getHelpOption();
        const helpRequested = helpOption && args.find((arg) => helpOption.is(arg));
        if (helpRequested) {
          this.outputHelp();
          this._exit(0, "commander.helpDisplayed", "(outputHelp)");
        }
      }
    };
    function incrementNodeInspectorPort(args) {
      return args.map((arg) => {
        if (!arg.startsWith("--inspect")) {
          return arg;
        }
        let debugOption;
        let debugHost = "127.0.0.1";
        let debugPort = "9229";
        let match;
        if ((match = arg.match(/^(--inspect(-brk)?)$/)) !== null) {
          debugOption = match[1];
        } else if ((match = arg.match(/^(--inspect(-brk|-port)?)=([^:]+)$/)) !== null) {
          debugOption = match[1];
          if (/^\d+$/.test(match[3])) {
            debugPort = match[3];
          } else {
            debugHost = match[3];
          }
        } else if ((match = arg.match(/^(--inspect(-brk|-port)?)=([^:]+):(\d+)$/)) !== null) {
          debugOption = match[1];
          debugHost = match[3];
          debugPort = match[4];
        }
        if (debugOption && debugPort !== "0") {
          return `${debugOption}=${debugHost}:${parseInt(debugPort) + 1}`;
        }
        return arg;
      });
    }
    function useColor() {
      if (process3.env.NO_COLOR || process3.env.FORCE_COLOR === "0" || process3.env.FORCE_COLOR === "false")
        return false;
      if (process3.env.FORCE_COLOR || process3.env.CLICOLOR_FORCE !== void 0)
        return true;
      return void 0;
    }
    exports.Command = Command2;
    exports.useColor = useColor;
  }
});

// node_modules/commander/index.js
var require_commander = __commonJS({
  "node_modules/commander/index.js"(exports) {
    var { Argument: Argument2 } = require_argument();
    var { Command: Command2 } = require_command();
    var { CommanderError: CommanderError2, InvalidArgumentError: InvalidArgumentError2 } = require_error();
    var { Help: Help2 } = require_help();
    var { Option: Option2 } = require_option();
    exports.program = new Command2();
    exports.createCommand = (name) => new Command2(name);
    exports.createOption = (flags, description) => new Option2(flags, description);
    exports.createArgument = (name, description) => new Argument2(name, description);
    exports.Command = Command2;
    exports.Option = Option2;
    exports.Argument = Argument2;
    exports.Help = Help2;
    exports.CommanderError = CommanderError2;
    exports.InvalidArgumentError = InvalidArgumentError2;
    exports.InvalidOptionArgumentError = InvalidArgumentError2;
  }
});

// node_modules/ms/index.js
var require_ms = __commonJS({
  "node_modules/ms/index.js"(exports, module) {
    var s = 1e3;
    var m = s * 60;
    var h2 = m * 60;
    var d = h2 * 24;
    var w = d * 7;
    var y2 = d * 365.25;
    module.exports = function(val, options) {
      options = options || {};
      var type = typeof val;
      if (type === "string" && val.length > 0) {
        return parse(val);
      } else if (type === "number" && isFinite(val)) {
        return options.long ? fmtLong(val) : fmtShort(val);
      }
      throw new Error(
        "val is not a non-empty string or a valid number. val=" + JSON.stringify(val)
      );
    };
    function parse(str) {
      str = String(str);
      if (str.length > 100) {
        return;
      }
      var match = /^(-?(?:\d+)?\.?\d+) *(milliseconds?|msecs?|ms|seconds?|secs?|s|minutes?|mins?|m|hours?|hrs?|h|days?|d|weeks?|w|years?|yrs?|y)?$/i.exec(
        str
      );
      if (!match) {
        return;
      }
      var n = parseFloat(match[1]);
      var type = (match[2] || "ms").toLowerCase();
      switch (type) {
        case "years":
        case "year":
        case "yrs":
        case "yr":
        case "y":
          return n * y2;
        case "weeks":
        case "week":
        case "w":
          return n * w;
        case "days":
        case "day":
        case "d":
          return n * d;
        case "hours":
        case "hour":
        case "hrs":
        case "hr":
        case "h":
          return n * h2;
        case "minutes":
        case "minute":
        case "mins":
        case "min":
        case "m":
          return n * m;
        case "seconds":
        case "second":
        case "secs":
        case "sec":
        case "s":
          return n * s;
        case "milliseconds":
        case "millisecond":
        case "msecs":
        case "msec":
        case "ms":
          return n;
        default:
          return void 0;
      }
    }
    function fmtShort(ms) {
      var msAbs = Math.abs(ms);
      if (msAbs >= d) {
        return Math.round(ms / d) + "d";
      }
      if (msAbs >= h2) {
        return Math.round(ms / h2) + "h";
      }
      if (msAbs >= m) {
        return Math.round(ms / m) + "m";
      }
      if (msAbs >= s) {
        return Math.round(ms / s) + "s";
      }
      return ms + "ms";
    }
    function fmtLong(ms) {
      var msAbs = Math.abs(ms);
      if (msAbs >= d) {
        return plural(ms, msAbs, d, "day");
      }
      if (msAbs >= h2) {
        return plural(ms, msAbs, h2, "hour");
      }
      if (msAbs >= m) {
        return plural(ms, msAbs, m, "minute");
      }
      if (msAbs >= s) {
        return plural(ms, msAbs, s, "second");
      }
      return ms + " ms";
    }
    function plural(ms, msAbs, n, name) {
      var isPlural = msAbs >= n * 1.5;
      return Math.round(ms / n) + " " + name + (isPlural ? "s" : "");
    }
  }
});

// node_modules/debug/src/common.js
var require_common = __commonJS({
  "node_modules/debug/src/common.js"(exports, module) {
    function setup(env2) {
      createDebug.debug = createDebug;
      createDebug.default = createDebug;
      createDebug.coerce = coerce;
      createDebug.disable = disable;
      createDebug.enable = enable;
      createDebug.enabled = enabled;
      createDebug.humanize = require_ms();
      createDebug.destroy = destroy;
      Object.keys(env2).forEach((key) => {
        createDebug[key] = env2[key];
      });
      createDebug.names = [];
      createDebug.skips = [];
      createDebug.formatters = {};
      function selectColor(namespace) {
        let hash = 0;
        for (let i2 = 0; i2 < namespace.length; i2++) {
          hash = (hash << 5) - hash + namespace.charCodeAt(i2);
          hash |= 0;
        }
        return createDebug.colors[Math.abs(hash) % createDebug.colors.length];
      }
      createDebug.selectColor = selectColor;
      function createDebug(namespace) {
        let prevTime;
        let enableOverride = null;
        let namespacesCache;
        let enabledCache;
        function debug2(...args) {
          if (!debug2.enabled) {
            return;
          }
          const self = debug2;
          const curr = Number(/* @__PURE__ */ new Date());
          const ms = curr - (prevTime || curr);
          self.diff = ms;
          self.prev = prevTime;
          self.curr = curr;
          prevTime = curr;
          args[0] = createDebug.coerce(args[0]);
          if (typeof args[0] !== "string") {
            args.unshift("%O");
          }
          let index = 0;
          args[0] = args[0].replace(/%([a-zA-Z%])/g, (match, format) => {
            if (match === "%%") {
              return "%";
            }
            index++;
            const formatter = createDebug.formatters[format];
            if (typeof formatter === "function") {
              const val = args[index];
              match = formatter.call(self, val);
              args.splice(index, 1);
              index--;
            }
            return match;
          });
          createDebug.formatArgs.call(self, args);
          const logFn = self.log || createDebug.log;
          logFn.apply(self, args);
        }
        debug2.namespace = namespace;
        debug2.useColors = createDebug.useColors();
        debug2.color = createDebug.selectColor(namespace);
        debug2.extend = extend;
        debug2.destroy = createDebug.destroy;
        Object.defineProperty(debug2, "enabled", {
          enumerable: true,
          configurable: false,
          get: () => {
            if (enableOverride !== null) {
              return enableOverride;
            }
            if (namespacesCache !== createDebug.namespaces) {
              namespacesCache = createDebug.namespaces;
              enabledCache = createDebug.enabled(namespace);
            }
            return enabledCache;
          },
          set: (v) => {
            enableOverride = v;
          }
        });
        if (typeof createDebug.init === "function") {
          createDebug.init(debug2);
        }
        return debug2;
      }
      function extend(namespace, delimiter) {
        const newDebug = createDebug(this.namespace + (typeof delimiter === "undefined" ? ":" : delimiter) + namespace);
        newDebug.log = this.log;
        return newDebug;
      }
      function enable(namespaces) {
        createDebug.save(namespaces);
        createDebug.namespaces = namespaces;
        createDebug.names = [];
        createDebug.skips = [];
        const split = (typeof namespaces === "string" ? namespaces : "").trim().replace(/\s+/g, ",").split(",").filter(Boolean);
        for (const ns of split) {
          if (ns[0] === "-") {
            createDebug.skips.push(ns.slice(1));
          } else {
            createDebug.names.push(ns);
          }
        }
      }
      function matchesTemplate(search, template) {
        let searchIndex = 0;
        let templateIndex = 0;
        let starIndex = -1;
        let matchIndex = 0;
        while (searchIndex < search.length) {
          if (templateIndex < template.length && (template[templateIndex] === search[searchIndex] || template[templateIndex] === "*")) {
            if (template[templateIndex] === "*") {
              starIndex = templateIndex;
              matchIndex = searchIndex;
              templateIndex++;
            } else {
              searchIndex++;
              templateIndex++;
            }
          } else if (starIndex !== -1) {
            templateIndex = starIndex + 1;
            matchIndex++;
            searchIndex = matchIndex;
          } else {
            return false;
          }
        }
        while (templateIndex < template.length && template[templateIndex] === "*") {
          templateIndex++;
        }
        return templateIndex === template.length;
      }
      function disable() {
        const namespaces = [
          ...createDebug.names,
          ...createDebug.skips.map((namespace) => "-" + namespace)
        ].join(",");
        createDebug.enable("");
        return namespaces;
      }
      function enabled(name) {
        for (const skip of createDebug.skips) {
          if (matchesTemplate(name, skip)) {
            return false;
          }
        }
        for (const ns of createDebug.names) {
          if (matchesTemplate(name, ns)) {
            return true;
          }
        }
        return false;
      }
      function coerce(val) {
        if (val instanceof Error) {
          return val.stack || val.message;
        }
        return val;
      }
      function destroy() {
        console.warn("Instance method `debug.destroy()` is deprecated and no longer does anything. It will be removed in the next major version of `debug`.");
      }
      createDebug.enable(createDebug.load());
      return createDebug;
    }
    module.exports = setup;
  }
});

// node_modules/debug/src/browser.js
var require_browser = __commonJS({
  "node_modules/debug/src/browser.js"(exports, module) {
    exports.formatArgs = formatArgs;
    exports.save = save;
    exports.load = load;
    exports.useColors = useColors;
    exports.storage = localstorage();
    exports.destroy = /* @__PURE__ */ (() => {
      let warned = false;
      return () => {
        if (!warned) {
          warned = true;
          console.warn("Instance method `debug.destroy()` is deprecated and no longer does anything. It will be removed in the next major version of `debug`.");
        }
      };
    })();
    exports.colors = [
      "#0000CC",
      "#0000FF",
      "#0033CC",
      "#0033FF",
      "#0066CC",
      "#0066FF",
      "#0099CC",
      "#0099FF",
      "#00CC00",
      "#00CC33",
      "#00CC66",
      "#00CC99",
      "#00CCCC",
      "#00CCFF",
      "#3300CC",
      "#3300FF",
      "#3333CC",
      "#3333FF",
      "#3366CC",
      "#3366FF",
      "#3399CC",
      "#3399FF",
      "#33CC00",
      "#33CC33",
      "#33CC66",
      "#33CC99",
      "#33CCCC",
      "#33CCFF",
      "#6600CC",
      "#6600FF",
      "#6633CC",
      "#6633FF",
      "#66CC00",
      "#66CC33",
      "#9900CC",
      "#9900FF",
      "#9933CC",
      "#9933FF",
      "#99CC00",
      "#99CC33",
      "#CC0000",
      "#CC0033",
      "#CC0066",
      "#CC0099",
      "#CC00CC",
      "#CC00FF",
      "#CC3300",
      "#CC3333",
      "#CC3366",
      "#CC3399",
      "#CC33CC",
      "#CC33FF",
      "#CC6600",
      "#CC6633",
      "#CC9900",
      "#CC9933",
      "#CCCC00",
      "#CCCC33",
      "#FF0000",
      "#FF0033",
      "#FF0066",
      "#FF0099",
      "#FF00CC",
      "#FF00FF",
      "#FF3300",
      "#FF3333",
      "#FF3366",
      "#FF3399",
      "#FF33CC",
      "#FF33FF",
      "#FF6600",
      "#FF6633",
      "#FF9900",
      "#FF9933",
      "#FFCC00",
      "#FFCC33"
    ];
    function useColors() {
      if (typeof window !== "undefined" && window.process && (window.process.type === "renderer" || window.process.__nwjs)) {
        return true;
      }
      if (typeof navigator !== "undefined" && navigator.userAgent && navigator.userAgent.toLowerCase().match(/(edge|trident)\/(\d+)/)) {
        return false;
      }
      let m;
      return typeof document !== "undefined" && document.documentElement && document.documentElement.style && document.documentElement.style.WebkitAppearance || // Is firebug? http://stackoverflow.com/a/398120/376773
      typeof window !== "undefined" && window.console && (window.console.firebug || window.console.exception && window.console.table) || // Is firefox >= v31?
      // https://developer.mozilla.org/en-US/docs/Tools/Web_Console#Styling_messages
      typeof navigator !== "undefined" && navigator.userAgent && (m = navigator.userAgent.toLowerCase().match(/firefox\/(\d+)/)) && parseInt(m[1], 10) >= 31 || // Double check webkit in userAgent just in case we are in a worker
      typeof navigator !== "undefined" && navigator.userAgent && navigator.userAgent.toLowerCase().match(/applewebkit\/(\d+)/);
    }
    function formatArgs(args) {
      args[0] = (this.useColors ? "%c" : "") + this.namespace + (this.useColors ? " %c" : " ") + args[0] + (this.useColors ? "%c " : " ") + "+" + module.exports.humanize(this.diff);
      if (!this.useColors) {
        return;
      }
      const c3 = "color: " + this.color;
      args.splice(1, 0, c3, "color: inherit");
      let index = 0;
      let lastC = 0;
      args[0].replace(/%[a-zA-Z%]/g, (match) => {
        if (match === "%%") {
          return;
        }
        index++;
        if (match === "%c") {
          lastC = index;
        }
      });
      args.splice(lastC, 0, c3);
    }
    exports.log = console.debug || console.log || (() => {
    });
    function save(namespaces) {
      try {
        if (namespaces) {
          exports.storage.setItem("debug", namespaces);
        } else {
          exports.storage.removeItem("debug");
        }
      } catch (error) {
      }
    }
    function load() {
      let r2;
      try {
        r2 = exports.storage.getItem("debug") || exports.storage.getItem("DEBUG");
      } catch (error) {
      }
      if (!r2 && typeof process !== "undefined" && "env" in process) {
        r2 = process.env.DEBUG;
      }
      return r2;
    }
    function localstorage() {
      try {
        return localStorage;
      } catch (error) {
      }
    }
    module.exports = require_common()(exports);
    var { formatters } = module.exports;
    formatters.j = function(v) {
      try {
        return JSON.stringify(v);
      } catch (error) {
        return "[UnexpectedJSONParseError]: " + error.message;
      }
    };
  }
});

// node_modules/has-flag/index.js
var require_has_flag = __commonJS({
  "node_modules/has-flag/index.js"(exports, module) {
    "use strict";
    module.exports = (flag, argv = process.argv) => {
      const prefix = flag.startsWith("-") ? "" : flag.length === 1 ? "-" : "--";
      const position = argv.indexOf(prefix + flag);
      const terminatorPosition = argv.indexOf("--");
      return position !== -1 && (terminatorPosition === -1 || position < terminatorPosition);
    };
  }
});

// node_modules/supports-color/index.js
var require_supports_color = __commonJS({
  "node_modules/supports-color/index.js"(exports, module) {
    "use strict";
    var os3 = __require("os");
    var tty2 = __require("tty");
    var hasFlag2 = require_has_flag();
    var { env: env2 } = process;
    var flagForceColor2;
    if (hasFlag2("no-color") || hasFlag2("no-colors") || hasFlag2("color=false") || hasFlag2("color=never")) {
      flagForceColor2 = 0;
    } else if (hasFlag2("color") || hasFlag2("colors") || hasFlag2("color=true") || hasFlag2("color=always")) {
      flagForceColor2 = 1;
    }
    function envForceColor2() {
      if ("FORCE_COLOR" in env2) {
        if (env2.FORCE_COLOR === "true") {
          return 1;
        }
        if (env2.FORCE_COLOR === "false") {
          return 0;
        }
        return env2.FORCE_COLOR.length === 0 ? 1 : Math.min(Number.parseInt(env2.FORCE_COLOR, 10), 3);
      }
    }
    function translateLevel2(level) {
      if (level === 0) {
        return false;
      }
      return {
        level,
        hasBasic: true,
        has256: level >= 2,
        has16m: level >= 3
      };
    }
    function supportsColor2(haveStream, { streamIsTTY, sniffFlags = true } = {}) {
      const noFlagForceColor = envForceColor2();
      if (noFlagForceColor !== void 0) {
        flagForceColor2 = noFlagForceColor;
      }
      const forceColor = sniffFlags ? flagForceColor2 : noFlagForceColor;
      if (forceColor === 0) {
        return 0;
      }
      if (sniffFlags) {
        if (hasFlag2("color=16m") || hasFlag2("color=full") || hasFlag2("color=truecolor")) {
          return 3;
        }
        if (hasFlag2("color=256")) {
          return 2;
        }
      }
      if (haveStream && !streamIsTTY && forceColor === void 0) {
        return 0;
      }
      const min = forceColor || 0;
      if (env2.TERM === "dumb") {
        return min;
      }
      if (process.platform === "win32") {
        const osRelease = os3.release().split(".");
        if (Number(osRelease[0]) >= 10 && Number(osRelease[2]) >= 10586) {
          return Number(osRelease[2]) >= 14931 ? 3 : 2;
        }
        return 1;
      }
      if ("CI" in env2) {
        if (["TRAVIS", "CIRCLECI", "APPVEYOR", "GITLAB_CI", "GITHUB_ACTIONS", "BUILDKITE", "DRONE"].some((sign) => sign in env2) || env2.CI_NAME === "codeship") {
          return 1;
        }
        return min;
      }
      if ("TEAMCITY_VERSION" in env2) {
        return /^(9\.(0*[1-9]\d*)\.|\d{2,}\.)/.test(env2.TEAMCITY_VERSION) ? 1 : 0;
      }
      if (env2.COLORTERM === "truecolor") {
        return 3;
      }
      if ("TERM_PROGRAM" in env2) {
        const version = Number.parseInt((env2.TERM_PROGRAM_VERSION || "").split(".")[0], 10);
        switch (env2.TERM_PROGRAM) {
          case "iTerm.app":
            return version >= 3 ? 3 : 2;
          case "Apple_Terminal":
            return 2;
        }
      }
      if (/-256(color)?$/i.test(env2.TERM)) {
        return 2;
      }
      if (/^screen|^xterm|^vt100|^vt220|^rxvt|color|ansi|cygwin|linux/i.test(env2.TERM)) {
        return 1;
      }
      if ("COLORTERM" in env2) {
        return 1;
      }
      return min;
    }
    function getSupportLevel(stream, options = {}) {
      const level = supportsColor2(stream, {
        streamIsTTY: stream && stream.isTTY,
        ...options
      });
      return translateLevel2(level);
    }
    module.exports = {
      supportsColor: getSupportLevel,
      stdout: getSupportLevel({ isTTY: tty2.isatty(1) }),
      stderr: getSupportLevel({ isTTY: tty2.isatty(2) })
    };
  }
});

// node_modules/debug/src/node.js
var require_node = __commonJS({
  "node_modules/debug/src/node.js"(exports, module) {
    var tty2 = __require("tty");
    var util = __require("util");
    exports.init = init;
    exports.log = log;
    exports.formatArgs = formatArgs;
    exports.save = save;
    exports.load = load;
    exports.useColors = useColors;
    exports.destroy = util.deprecate(
      () => {
      },
      "Instance method `debug.destroy()` is deprecated and no longer does anything. It will be removed in the next major version of `debug`."
    );
    exports.colors = [6, 2, 3, 4, 5, 1];
    try {
      const supportsColor2 = require_supports_color();
      if (supportsColor2 && (supportsColor2.stderr || supportsColor2).level >= 2) {
        exports.colors = [
          20,
          21,
          26,
          27,
          32,
          33,
          38,
          39,
          40,
          41,
          42,
          43,
          44,
          45,
          56,
          57,
          62,
          63,
          68,
          69,
          74,
          75,
          76,
          77,
          78,
          79,
          80,
          81,
          92,
          93,
          98,
          99,
          112,
          113,
          128,
          129,
          134,
          135,
          148,
          149,
          160,
          161,
          162,
          163,
          164,
          165,
          166,
          167,
          168,
          169,
          170,
          171,
          172,
          173,
          178,
          179,
          184,
          185,
          196,
          197,
          198,
          199,
          200,
          201,
          202,
          203,
          204,
          205,
          206,
          207,
          208,
          209,
          214,
          215,
          220,
          221
        ];
      }
    } catch (error) {
    }
    exports.inspectOpts = Object.keys(process.env).filter((key) => {
      return /^debug_/i.test(key);
    }).reduce((obj, key) => {
      const prop = key.substring(6).toLowerCase().replace(/_([a-z])/g, (_2, k2) => {
        return k2.toUpperCase();
      });
      let val = process.env[key];
      if (/^(yes|on|true|enabled)$/i.test(val)) {
        val = true;
      } else if (/^(no|off|false|disabled)$/i.test(val)) {
        val = false;
      } else if (val === "null") {
        val = null;
      } else {
        val = Number(val);
      }
      obj[prop] = val;
      return obj;
    }, {});
    function useColors() {
      return "colors" in exports.inspectOpts ? Boolean(exports.inspectOpts.colors) : tty2.isatty(process.stderr.fd);
    }
    function formatArgs(args) {
      const { namespace: name, useColors: useColors2 } = this;
      if (useColors2) {
        const c3 = this.color;
        const colorCode = "\x1B[3" + (c3 < 8 ? c3 : "8;5;" + c3);
        const prefix = `  ${colorCode};1m${name} \x1B[0m`;
        args[0] = prefix + args[0].split("\n").join("\n" + prefix);
        args.push(colorCode + "m+" + module.exports.humanize(this.diff) + "\x1B[0m");
      } else {
        args[0] = getDate() + name + " " + args[0];
      }
    }
    function getDate() {
      if (exports.inspectOpts.hideDate) {
        return "";
      }
      return (/* @__PURE__ */ new Date()).toISOString() + " ";
    }
    function log(...args) {
      return process.stderr.write(util.formatWithOptions(exports.inspectOpts, ...args) + "\n");
    }
    function save(namespaces) {
      if (namespaces) {
        process.env.DEBUG = namespaces;
      } else {
        delete process.env.DEBUG;
      }
    }
    function load() {
      return process.env.DEBUG;
    }
    function init(debug2) {
      debug2.inspectOpts = {};
      const keys = Object.keys(exports.inspectOpts);
      for (let i2 = 0; i2 < keys.length; i2++) {
        debug2.inspectOpts[keys[i2]] = exports.inspectOpts[keys[i2]];
      }
    }
    module.exports = require_common()(exports);
    var { formatters } = module.exports;
    formatters.o = function(v) {
      this.inspectOpts.colors = this.useColors;
      return util.inspect(v, this.inspectOpts).split("\n").map((str) => str.trim()).join(" ");
    };
    formatters.O = function(v) {
      this.inspectOpts.colors = this.useColors;
      return util.inspect(v, this.inspectOpts);
    };
  }
});

// node_modules/debug/src/index.js
var require_src = __commonJS({
  "node_modules/debug/src/index.js"(exports, module) {
    if (typeof process === "undefined" || process.type === "renderer" || process.browser === true || process.__nwjs) {
      module.exports = require_browser();
    } else {
      module.exports = require_node();
    }
  }
});

// node_modules/@kwsites/file-exists/dist/src/index.js
var require_src2 = __commonJS({
  "node_modules/@kwsites/file-exists/dist/src/index.js"(exports) {
    "use strict";
    var __importDefault = exports && exports.__importDefault || function(mod) {
      return mod && mod.__esModule ? mod : { "default": mod };
    };
    Object.defineProperty(exports, "__esModule", { value: true });
    var fs_1 = __require("fs");
    var debug_1 = __importDefault(require_src());
    var log = debug_1.default("@kwsites/file-exists");
    function check(path2, isFile, isDirectory) {
      log(`checking %s`, path2);
      try {
        const stat = fs_1.statSync(path2);
        if (stat.isFile() && isFile) {
          log(`[OK] path represents a file`);
          return true;
        }
        if (stat.isDirectory() && isDirectory) {
          log(`[OK] path represents a directory`);
          return true;
        }
        log(`[FAIL] path represents something other than a file or directory`);
        return false;
      } catch (e) {
        if (e.code === "ENOENT") {
          log(`[FAIL] path is not accessible: %o`, e);
          return false;
        }
        log(`[FATAL] %o`, e);
        throw e;
      }
    }
    function exists2(path2, type = exports.READABLE) {
      return check(path2, (type & exports.FILE) > 0, (type & exports.FOLDER) > 0);
    }
    exports.exists = exists2;
    exports.FILE = 1;
    exports.FOLDER = 2;
    exports.READABLE = exports.FILE + exports.FOLDER;
  }
});

// node_modules/@kwsites/file-exists/dist/index.js
var require_dist = __commonJS({
  "node_modules/@kwsites/file-exists/dist/index.js"(exports) {
    "use strict";
    function __export3(m) {
      for (var p2 in m) if (!exports.hasOwnProperty(p2)) exports[p2] = m[p2];
    }
    Object.defineProperty(exports, "__esModule", { value: true });
    __export3(require_src2());
  }
});

// node_modules/@simple-git/args-pathspec/dist/index.mjs
function c(...n) {
  const e = new String(n);
  return t.set(e, n), e;
}
function r(n) {
  return n instanceof String && t.has(n);
}
function o(n) {
  return t.get(n) ?? [];
}
var t;
var init_dist = __esm({
  "node_modules/@simple-git/args-pathspec/dist/index.mjs"() {
    t = /* @__PURE__ */ new WeakMap();
  }
});

// node_modules/@kwsites/promise-deferred/dist/index.js
var require_dist2 = __commonJS({
  "node_modules/@kwsites/promise-deferred/dist/index.js"(exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.createDeferred = exports.deferred = void 0;
    function deferred2() {
      let done;
      let fail;
      let status = "pending";
      const promise = new Promise((_done, _fail) => {
        done = _done;
        fail = _fail;
      });
      return {
        promise,
        done(result) {
          if (status === "pending") {
            status = "resolved";
            done(result);
          }
        },
        fail(error) {
          if (status === "pending") {
            status = "rejected";
            fail(error);
          }
        },
        get fulfilled() {
          return status !== "pending";
        },
        get status() {
          return status;
        }
      };
    }
    exports.deferred = deferred2;
    exports.createDeferred = deferred2;
    exports.default = deferred2;
  }
});

// node_modules/@simple-git/argv-parser/dist/index.mjs
function* U(e, t2) {
  const n = t2 === "global";
  for (const o2 of e)
    o2.isGlobal === n && (yield o2);
}
function F(e, t2) {
  for (const { name: o2 } of U(e, "task")) {
    if (k.has(o2))
      return p(true, t2);
    if (S.has(o2))
      return p(false, t2);
  }
  const n = t2.at(0)?.toLowerCase();
  return n === void 0 ? null : P.has(n) ? p(true, t2.slice(1)) : E.has(n) ? p(false, t2.slice(1)) : t2.length === 1 ? p(false, t2) : p(true, t2);
}
function p(e = false, t2 = []) {
  const n = t2.at(0)?.toLowerCase();
  return n === void 0 ? null : {
    isWrite: e,
    isRead: !e,
    key: n,
    value: t2.at(1)
  };
}
function A(e, t2) {
  return t2.isWrite && t2.value !== void 0 ? { key: t2.key, value: t2.value, scope: e } : { key: t2.key, scope: e };
}
function M(e) {
  const t2 = e?.indexOf("=") || -1;
  return !e || t2 < 0 ? null : {
    key: e.slice(0, t2).trim().toLowerCase(),
    value: e.slice(t2 + 1)
  };
}
function N(e) {
  for (const { name: t2 } of U(e, "task"))
    switch (t2) {
      case "--global":
        return "global";
      case "--system":
        return "system";
      case "--worktree":
        return "worktree";
      case "--local":
        return "local";
      case "--file":
      case "-f":
        return "file";
    }
  return "local";
}
function G({ name: e }) {
  if (e === "-c" || e === "--config")
    return "inline";
  if (e === "--config-env")
    return "env";
}
function* O(e) {
  for (const t2 of e) {
    const n = G(t2), o2 = n && M(t2.value);
    o2 && (yield {
      ...o2,
      scope: n
    });
  }
}
function L(e, t2, n) {
  const o2 = {
    read: [],
    write: [...O(t2)]
  };
  return e === "config" && $(
    o2,
    N(t2),
    F(t2, n)
  ), o2;
}
function $(e, t2, n) {
  if (n === null)
    return;
  const o2 = A(t2, n);
  n.isWrite ? e.write.push(o2) : e.read.push(o2);
}
function I(e) {
  const t2 = R[e ?? ""] ?? T;
  return {
    short: new Map([...x.short.entries(), ...t2.short.entries()]),
    long: t2.long
  };
}
function b(e, t2 = D) {
  if (e.startsWith("--")) {
    const n = e.indexOf("=");
    if (n > 2)
      return [{ name: e.slice(0, n), value: e.slice(n + 1), needsNext: false }];
    const o2 = e.slice(2);
    return [{ name: e, needsNext: t2.long.has(o2) }];
  }
  if (e.length === 2) {
    const n = e.charAt(1), o2 = t2.short.get(n);
    return [{ name: e, needsNext: o2 === true }];
  }
  return W(e, t2.short);
}
function W(e, t2) {
  const n = e.slice(1).split(""), o2 = [];
  for (let s = 0; s < n.length; s++) {
    const r2 = n[s], l = t2.get(r2);
    if (l === void 0)
      return [{ name: e, needsNext: false }];
    if (l) {
      const a = n.slice(s + 1).join("");
      if (a && ![...a].every((w) => t2.has(w)))
        return o2.push({ name: `-${r2}`, value: a, needsNext: false }), o2;
    }
    o2.push({ name: `-${r2}`, needsNext: l });
  }
  return o2;
}
function j(e, t2 = []) {
  let n = 0;
  for (; n < e.length; ) {
    const o2 = String(e[n]);
    if (!o2.startsWith("-") || o2.length < 2) break;
    const s = b(o2);
    let r2 = n + 1;
    for (const l of s) {
      const a = {
        name: l.name,
        value: l.value,
        absorbedNext: false,
        isGlobal: true
      };
      l.needsNext && a.value === void 0 && r2 < e.length && (a.value = String(e[r2]), a.absorbedNext = true, r2++), t2.push(a);
    }
    n = r2;
  }
  return { flags: t2, taskIndex: n };
}
function B(e, t2, n = []) {
  const o2 = I(t2), s = [], r2 = [];
  let l = 0;
  for (; l < e.length; ) {
    const a = e[l];
    if (r(a)) {
      r2.push(...o(a)), l++;
      continue;
    }
    const f = String(a);
    if (f === "--") {
      for (let g = l + 1; g < e.length; g++) {
        const u = e[g];
        r(u) ? r2.push(...o(u)) : r2.push(String(u));
      }
      break;
    }
    if (!f.startsWith("-") || f.length < 2) {
      s.push(f), l++;
      continue;
    }
    const w = b(f, o2);
    let d = l + 1;
    for (const g of w) {
      const u = {
        name: g.name,
        value: g.value,
        absorbedNext: false,
        isGlobal: false
      };
      g.needsNext && u.value === void 0 && d < e.length && !r(e[d]) && (u.value = String(e[d]), u.absorbedNext = true, d++), n.push(u);
    }
    l = d;
  }
  return { flags: n, positionals: s, pathspecs: r2 };
}
function* V({
  write: e
}) {
  for (const t2 of e)
    for (const n of q) {
      const o2 = n(t2.key);
      o2 && (yield o2);
    }
}
function c2(e, t2, n = String(e)) {
  const o2 = typeof e == "string" ? new RegExp(`\\s*${e.toLowerCase()}`) : e;
  return function(r2) {
    if (o2.test(r2))
      return {
        category: t2,
        message: `Configuring ${n} is not permitted without enabling ${t2}`
      };
  };
}
function i(e, t2) {
  const n = new RegExp(`\\s*${e.toLowerCase().replace(/\./g, "(..+)?.")}`);
  return c2(n, t2, e);
}
function* K(e, t2) {
  for (const n of t2)
    for (const o2 of H) {
      const s = o2(e, n.name);
      s && (yield s);
    }
}
function h(e, t2, n, o2 = String(t2)) {
  const s = typeof t2 == "string" ? new RegExp(`\\s*${t2.toLowerCase()}`) : t2, r2 = `Use of ${e ? `${e} with option ` : ""}${o2} is not permitted without enabling ${n}`;
  return function(a, f) {
    if ((!e || a === e) && s.test(f))
      return {
        category: n,
        message: r2
      };
  };
}
function C(e, t2, n) {
  return [...K(e, t2), ...V(n)];
}
function Y(...e) {
  const { flags: t2, taskIndex: n } = j(e), o2 = n < e.length ? String(e[n]).toLowerCase() : null, s = o2 !== null ? e.slice(n + 1) : [], { positionals: r2, pathspecs: l } = B(s, o2, t2), a = L(o2, t2, r2);
  return {
    task: o2,
    flags: t2.map(J),
    paths: l,
    config: a,
    vulnerabilities: z(C(o2, t2, a))
  };
}
function z(e) {
  return Object.defineProperty(e, "vulnerabilities", {
    value: e
  });
}
function J({ value: e, name: t2 }) {
  return e !== void 0 ? { name: t2, value: e } : { name: t2 };
}
function* Q(e) {
  const t2 = parseInt(e.git_config_count ?? "0", 10);
  for (let n = 0; n < t2; n++) {
    const o2 = e[`git_config_key_${n}`], s = e[`git_config_value_${n}`];
    o2 !== void 0 && (yield { key: o2.toLowerCase().trim(), value: s, scope: "env" });
  }
}
function* X(e) {
  for (const t2 of Object.keys(e))
    if (_(t2)) {
      const n = y[t2];
      yield {
        category: n,
        message: `Use of "${t2.toUpperCase()}" is not permitted without enabling ${n}`
      };
    }
}
function _(e) {
  return Object.hasOwn(y, e);
}
function Z(e) {
  const t2 = {};
  for (const [n, o2] of Object.entries(e)) {
    const s = n.toLowerCase().trim();
    (_(s) || s.startsWith("git")) && (t2[s] = String(o2));
  }
  return t2;
}
function ee(e) {
  const t2 = Z(e), n = {
    read: [],
    write: [...Q(t2)]
  }, o2 = [
    ...X(t2),
    ...C(null, [], n)
  ];
  return {
    config: n,
    vulnerabilities: o2
  };
}
function ne(e, t2) {
  return [...Y(...e).vulnerabilities, ...ee(t2).vulnerabilities];
}
var k, S, P, E, x, D, R, T, q, H, y;
var init_dist2 = __esm({
  "node_modules/@simple-git/argv-parser/dist/index.mjs"() {
    init_dist();
    k = /* @__PURE__ */ new Set([
      "--add",
      "--edit",
      "--remove-section",
      "--rename-section",
      "--replace-all",
      "--unset",
      "--unset-all",
      "-e"
    ]);
    S = /* @__PURE__ */ new Set([
      "--get",
      "--get-all",
      "--get-color",
      "--get-colorbool",
      "--get-regexp",
      "--get-urlmatch",
      "--list",
      "-l"
    ]);
    P = /* @__PURE__ */ new Set([
      "edit",
      "remove-section",
      "rename-section",
      "set",
      "unset"
    ]);
    E = /* @__PURE__ */ new Set(["get", "get-color", "get-colorbool", "list"]);
    x = {
      short: /* @__PURE__ */ new Map([
        ["c", true]
        //  -c <k=v>    set config key for this invocation
      ])
    };
    D = {
      short: new Map([
        ["C", true],
        //  -C <path>   change working directory
        ["P", false],
        // -P          no pager (alias for --no-pager)
        ["h", false],
        // -h          help
        ["p", false],
        // -p          paginate
        ["v", false],
        // -v          version
        ...x.short.entries()
      ]),
      long: /* @__PURE__ */ new Set([
        "attr-source",
        "config-env",
        "exec-path",
        "git-dir",
        "list-cmds",
        "namespace",
        "super-prefix",
        "work-tree"
      ])
    };
    R = {
      clone: {
        short: /* @__PURE__ */ new Map([
          ["b", true],
          // -b <branch>
          ["j", true],
          // -j <n>          parallel jobs
          ["l", false],
          // -l local
          ["n", false],
          // -n no-checkout
          ["o", true],
          // -o <name>       remote name
          ["q", false],
          // -q quiet
          ["s", false],
          // -s shared
          ["u", true]
          // -u <upload-pack>
        ]),
        long: /* @__PURE__ */ new Set(["branch", "config", "jobs", "origin", "upload-pack", "u", "template"])
      },
      commit: {
        short: /* @__PURE__ */ new Map([
          ["C", true],
          // -C <commit>  reuse message
          ["F", true],
          // -F <file>    read message from file
          ["c", true],
          // -c <commit>  reedit message
          ["m", true],
          // -m <msg>
          ["t", true]
          // -t <template>
        ]),
        long: /* @__PURE__ */ new Set(["file", "message", "reedit-message", "reuse-message", "template"])
      },
      config: {
        short: /* @__PURE__ */ new Map([
          ["e", false],
          // -e  open editor
          ["f", true],
          //  -f <file>
          ["l", false]
          // -l  list
        ]),
        long: /* @__PURE__ */ new Set(["blob", "comment", "default", "file", "type", "value"])
      },
      fetch: {
        short: /* @__PURE__ */ new Map(),
        long: /* @__PURE__ */ new Set(["upload-pack"])
      },
      init: {
        short: /* @__PURE__ */ new Map(),
        long: /* @__PURE__ */ new Set(["template"])
      },
      pull: {
        short: /* @__PURE__ */ new Map(),
        long: /* @__PURE__ */ new Set(["upload-pack"])
      },
      push: {
        short: /* @__PURE__ */ new Map(),
        long: /* @__PURE__ */ new Set(["exec", "receive-pack"])
      }
    };
    T = { short: /* @__PURE__ */ new Map(), long: /* @__PURE__ */ new Set() };
    q = [
      c2("alias", "allowUnsafeAlias"),
      c2("core.askPass", "allowUnsafeAskPass"),
      c2("core.editor", "allowUnsafeEditor"),
      c2("core.fsmonitor", "allowUnsafeFsMonitor"),
      c2("core.gitProxy", "allowUnsafeGitProxy"),
      c2("core.hooksPath", "allowUnsafeHooksPath"),
      c2("core.pager", "allowUnsafePager"),
      c2("core.sshCommand", "allowUnsafeSshCommand"),
      i("credential.helper", "allowUnsafeCredentialHelper"),
      i("diff.command", "allowUnsafeDiffExternal"),
      c2("diff.external", "allowUnsafeDiffExternal"),
      i("diff.textconv", "allowUnsafeDiffTextConv"),
      i("filter.clean", "allowUnsafeFilter"),
      i("filter.smudge", "allowUnsafeFilter"),
      i("gpg.program", "allowUnsafeGpgProgram"),
      c2("init.templateDir", "allowUnsafeTemplateDir"),
      i("merge.driver", "allowUnsafeMergeDriver"),
      i("mergetool.path", "allowUnsafeMergeDriver"),
      i("mergetool.cmd", "allowUnsafeMergeDriver"),
      i("protocol.allow", "allowUnsafeProtocolOverride"),
      i("remote.receivepack", "allowUnsafePack"),
      i("remote.uploadpack", "allowUnsafePack"),
      c2("sequence.editor", "allowUnsafeEditor")
    ];
    H = [
      h(
        null,
        /--(upload|receive)-pack/,
        "allowUnsafePack",
        "--upload-pack or --receive-pack"
      ),
      h("clone", /^-\w*u/, "allowUnsafePack"),
      h("clone", "--u", "allowUnsafePack"),
      h("push", "--exec", "allowUnsafePack"),
      h(null, "--template", "allowUnsafeTemplateDir")
    ];
    y = {
      editor: "allowUnsafeEditor",
      git_askpass: "allowUnsafeAskPass",
      git_config_global: "allowUnsafeConfigPaths",
      git_config_system: "allowUnsafeConfigPaths",
      git_config_count: "allowUnsafeConfigEnvCount",
      git_config: "allowUnsafeConfigPaths",
      git_editor: "allowUnsafeEditor",
      git_exec_path: "allowUnsafeConfigPaths",
      git_external_diff: "allowUnsafeDiffExternal",
      git_pager: "allowUnsafePager",
      git_proxy_command: "allowUnsafeGitProxy",
      git_template_dir: "allowUnsafeTemplateDir",
      git_sequence_editor: "allowUnsafeEditor",
      git_ssh: "allowUnsafeSshCommand",
      git_ssh_command: "allowUnsafeSshCommand",
      pager: "allowUnsafePager",
      prefix: "allowUnsafeConfigPaths",
      ssh_askpass: "allowUnsafeAskPass"
    };
  }
});

// node_modules/simple-git/dist/esm/index.js
var esm_exports = {};
__export(esm_exports, {
  CheckRepoActions: () => CheckRepoActions,
  CleanOptions: () => CleanOptions,
  DiffNameStatus: () => DiffNameStatus,
  GitConfigScope: () => GitConfigScope,
  GitConstructError: () => GitConstructError,
  GitError: () => GitError,
  GitPluginError: () => GitPluginError,
  GitResponseError: () => GitResponseError,
  ResetMode: () => ResetMode,
  TaskConfigurationError: () => TaskConfigurationError,
  default: () => esm_default,
  gitP: () => gitP,
  grepQueryBuilder: () => grepQueryBuilder,
  pathspec: () => c,
  simpleGit: () => simpleGit
});
import { spawn } from "child_process";
import { normalize } from "node:path";
import { EventEmitter } from "node:events";
function asFunction(source) {
  if (typeof source !== "function") {
    return NOOP;
  }
  return source;
}
function isUserFunction(source) {
  return typeof source === "function" && source !== NOOP;
}
function splitOn(input, char) {
  const index = input.indexOf(char);
  if (index <= 0) {
    return [input, ""];
  }
  return [input.substr(0, index), input.substr(index + 1)];
}
function first(input, offset = 0) {
  return isArrayLike(input) && input.length > offset ? input[offset] : void 0;
}
function last(input, offset = 0) {
  if (isArrayLike(input) && input.length > offset) {
    return input[input.length - 1 - offset];
  }
}
function isArrayLike(input) {
  return filterHasLength(input);
}
function toLinesWithContent(input = "", trimmed2 = true, separator = "\n") {
  return input.split(separator).reduce((output, line) => {
    const lineContent = trimmed2 ? line.trim() : line;
    if (lineContent) {
      output.push(lineContent);
    }
    return output;
  }, []);
}
function forEachLineWithContent(input, callback) {
  return toLinesWithContent(input, true).map((line) => callback(line));
}
function folderExists(path2) {
  return (0, import_file_exists.exists)(path2, import_file_exists.FOLDER);
}
function append(target, item) {
  if (Array.isArray(target)) {
    if (!target.includes(item)) {
      target.push(item);
    }
  } else {
    target.add(item);
  }
  return item;
}
function including(target, item) {
  if (Array.isArray(target) && !target.includes(item)) {
    target.push(item);
  }
  return target;
}
function remove(target, item) {
  if (Array.isArray(target)) {
    const index = target.indexOf(item);
    if (index >= 0) {
      target.splice(index, 1);
    }
  } else {
    target.delete(item);
  }
  return item;
}
function asArray(source) {
  return Array.isArray(source) ? source : [source];
}
function asCamelCase(str) {
  return str.replace(/[\s-]+(.)/g, (_all, chr) => {
    return chr.toUpperCase();
  });
}
function asStringArray(source) {
  return asArray(source).map((item) => {
    return item instanceof String ? item : String(item);
  });
}
function asNumber(source, onNaN = 0) {
  if (source == null) {
    return onNaN;
  }
  const num = parseInt(source, 10);
  return Number.isNaN(num) ? onNaN : num;
}
function prefixedArray(input, prefix) {
  const output = [];
  for (let i2 = 0, max = input.length; i2 < max; i2++) {
    output.push(prefix, input[i2]);
  }
  return output;
}
function bufferToString(input) {
  return (Array.isArray(input) ? Buffer.concat(input) : input).toString("utf-8");
}
function pick(source, properties) {
  const out = {};
  properties.forEach((key) => {
    if (source[key] !== void 0) {
      out[key] = source[key];
    }
  });
  return out;
}
function delay(duration = 0) {
  return new Promise((done) => setTimeout(done, duration));
}
function orVoid(input) {
  if (input === false) {
    return void 0;
  }
  return input;
}
function filterType(input, filter, def) {
  if (filter(input)) {
    return input;
  }
  return arguments.length > 2 ? def : void 0;
}
function filterPrimitives(input, omit) {
  const type = r(input) ? "string" : typeof input;
  return /number|string|boolean/.test(type) && (!omit || !omit.includes(type));
}
function filterPlainObject(input) {
  return !!input && objectToString(input) === "[object Object]";
}
function filterFunction(input) {
  return typeof input === "function";
}
function useMatchesDefault() {
  throw new Error(`LineParser:useMatches not implemented`);
}
function createInstanceConfig(...options) {
  const baseDir = process.cwd();
  const config = Object.assign(
    { baseDir, ...defaultOptions },
    ...options.filter((o2) => typeof o2 === "object" && o2)
  );
  config.baseDir = config.baseDir || baseDir;
  config.trimmed = config.trimmed === true;
  return config;
}
function appendTaskOptions(options, commands = []) {
  if (!filterPlainObject(options)) {
    return commands;
  }
  return Object.keys(options).reduce((commands2, key) => {
    const value = options[key];
    if (r(value)) {
      commands2.push(value);
    } else if (filterPrimitives(value, ["boolean"])) {
      commands2.push(key + "=" + value);
    } else if (Array.isArray(value)) {
      for (const v of value) {
        if (!filterPrimitives(v, ["string", "number"])) {
          commands2.push(key + "=" + v);
        }
      }
    } else {
      commands2.push(key);
    }
    return commands2;
  }, commands);
}
function getTrailingOptions(args, initialPrimitive = 0, objectOnly = false) {
  const command = [];
  for (let i2 = 0, max = initialPrimitive < 0 ? args.length : initialPrimitive; i2 < max; i2++) {
    if ("string|number".includes(typeof args[i2])) {
      command.push(String(args[i2]));
    }
  }
  appendTaskOptions(trailingOptionsArgument(args), command);
  if (!objectOnly) {
    command.push(...trailingArrayArgument(args));
  }
  return command;
}
function trailingArrayArgument(args) {
  const hasTrailingCallback = typeof last(args) === "function";
  return asStringArray(filterType(last(args, hasTrailingCallback ? 1 : 0), filterArray, []));
}
function trailingOptionsArgument(args) {
  const hasTrailingCallback = filterFunction(last(args));
  return filterType(last(args, hasTrailingCallback ? 1 : 0), filterPlainObject);
}
function trailingFunctionArgument(args, includeNoop = true) {
  const callback = asFunction(last(args));
  return includeNoop || isUserFunction(callback) ? callback : void 0;
}
function callTaskParser(parser4, streams) {
  return parser4(streams.stdOut, streams.stdErr);
}
function parseStringResponse(result, parsers12, texts, trim = true) {
  asArray(texts).forEach((text) => {
    for (let lines = toLinesWithContent(text, trim), i2 = 0, max = lines.length; i2 < max; i2++) {
      const line = (offset = 0) => {
        if (i2 + offset >= max) {
          return;
        }
        return lines[i2 + offset];
      };
      parsers12.some(({ parse }) => parse(line, result));
    }
  });
  return result;
}
function checkIsRepoTask(action) {
  switch (action) {
    case "bare":
      return checkIsBareRepoTask();
    case "root":
      return checkIsRepoRootTask();
  }
  const commands = ["rev-parse", "--is-inside-work-tree"];
  return {
    commands,
    format: "utf-8",
    onError,
    parser
  };
}
function checkIsRepoRootTask() {
  const commands = ["rev-parse", "--git-dir"];
  return {
    commands,
    format: "utf-8",
    onError,
    parser(path2) {
      return /^\.(git)?$/.test(path2.trim());
    }
  };
}
function checkIsBareRepoTask() {
  const commands = ["rev-parse", "--is-bare-repository"];
  return {
    commands,
    format: "utf-8",
    onError,
    parser
  };
}
function isNotRepoMessage(error) {
  return /(Not a git repository|Kein Git-Repository)/i.test(String(error));
}
function cleanSummaryParser(dryRun, text) {
  const summary = new CleanResponse(dryRun);
  const regexp = dryRun ? dryRunRemovalRegexp : removalRegexp;
  toLinesWithContent(text).forEach((line) => {
    const removed = line.replace(regexp, "");
    summary.paths.push(removed);
    (isFolderRegexp.test(removed) ? summary.folders : summary.files).push(removed);
  });
  return summary;
}
function adhocExecTask(parser4) {
  return {
    commands: EMPTY_COMMANDS,
    format: "empty",
    parser: parser4
  };
}
function configurationErrorTask(error) {
  return {
    commands: EMPTY_COMMANDS,
    format: "empty",
    parser() {
      throw typeof error === "string" ? new TaskConfigurationError(error) : error;
    }
  };
}
function straightThroughStringTask(commands, trimmed2 = false) {
  return {
    commands,
    format: "utf-8",
    parser(text) {
      return trimmed2 ? String(text).trim() : text;
    }
  };
}
function straightThroughBufferTask(commands) {
  return {
    commands,
    format: "buffer",
    parser(buffer) {
      return buffer;
    }
  };
}
function isBufferTask(task) {
  return task.format === "buffer";
}
function isEmptyTask(task) {
  return task.format === "empty" || !task.commands.length;
}
function cleanWithOptionsTask(mode, customArgs) {
  const { cleanMode, options, valid } = getCleanOptions(mode);
  if (!cleanMode) {
    return configurationErrorTask(CONFIG_ERROR_MODE_REQUIRED);
  }
  if (!valid.options) {
    return configurationErrorTask(CONFIG_ERROR_UNKNOWN_OPTION + JSON.stringify(mode));
  }
  options.push(...customArgs);
  if (options.some(isInteractiveMode)) {
    return configurationErrorTask(CONFIG_ERROR_INTERACTIVE_MODE);
  }
  return cleanTask(cleanMode, options);
}
function cleanTask(mode, customArgs) {
  const commands = ["clean", `-${mode}`, ...customArgs];
  return {
    commands,
    format: "utf-8",
    parser(text) {
      return cleanSummaryParser(mode === "n", text);
    }
  };
}
function isCleanOptionsArray(input) {
  return Array.isArray(input) && input.every((test) => CleanOptionValues.has(test));
}
function getCleanOptions(input) {
  let cleanMode;
  let options = [];
  let valid = { cleanMode: false, options: true };
  input.replace(/[^a-z]i/g, "").split("").forEach((char) => {
    if (isCleanMode(char)) {
      cleanMode = char;
      valid.cleanMode = true;
    } else {
      valid.options = valid.options && isKnownOption(options[options.length] = `-${char}`);
    }
  });
  return {
    cleanMode,
    options,
    valid
  };
}
function isCleanMode(cleanMode) {
  return cleanMode === "f" || cleanMode === "n";
}
function isKnownOption(option) {
  return /^-[a-z]$/i.test(option) && CleanOptionValues.has(option.charAt(1));
}
function isInteractiveMode(option) {
  if (/^-[^\-]/.test(option)) {
    return option.indexOf("i") > 0;
  }
  return option === "--interactive";
}
function configListParser(text) {
  const config = new ConfigList();
  for (const item of configParser(text)) {
    config.addValue(item.file, String(item.key), item.value);
  }
  return config;
}
function configGetParser(text, key) {
  let value = null;
  const values = [];
  const scopes = /* @__PURE__ */ new Map();
  for (const item of configParser(text, key)) {
    if (item.key !== key) {
      continue;
    }
    values.push(value = item.value);
    if (!scopes.has(item.file)) {
      scopes.set(item.file, []);
    }
    scopes.get(item.file).push(value);
  }
  return {
    key,
    paths: Array.from(scopes.keys()),
    scopes,
    value,
    values
  };
}
function configFilePath(filePath) {
  return filePath.replace(/^(file):/, "");
}
function* configParser(text, requestedKey = null) {
  const lines = text.split("\0");
  for (let i2 = 0, max = lines.length - 1; i2 < max; ) {
    const file = configFilePath(lines[i2++]);
    let value = lines[i2++];
    let key = requestedKey;
    if (value.includes("\n")) {
      const line = splitOn(value, "\n");
      key = line[0];
      value = line[1];
    }
    yield { file, key, value };
  }
}
function asConfigScope(scope, fallback) {
  if (typeof scope === "string" && Object.hasOwn(GitConfigScope, scope)) {
    return scope;
  }
  return fallback;
}
function addConfigTask(key, value, append2, scope) {
  const commands = ["config", `--${scope}`];
  if (append2) {
    commands.push("--add");
  }
  commands.push(key, value);
  return {
    commands,
    format: "utf-8",
    parser(text) {
      return text;
    }
  };
}
function getConfigTask(key, scope) {
  const commands = ["config", "--null", "--show-origin", "--get-all", key];
  if (scope) {
    commands.splice(1, 0, `--${scope}`);
  }
  return {
    commands,
    format: "utf-8",
    parser(text) {
      return configGetParser(text, key);
    }
  };
}
function listConfigTask(scope) {
  const commands = ["config", "--list", "--show-origin", "--null"];
  if (scope) {
    commands.push(`--${scope}`);
  }
  return {
    commands,
    format: "utf-8",
    parser(text) {
      return configListParser(text);
    }
  };
}
function config_default() {
  return {
    addConfig(key, value, ...rest) {
      return this._runTask(
        addConfigTask(
          key,
          value,
          rest[0] === true,
          asConfigScope(
            rest[1],
            "local"
            /* local */
          )
        ),
        trailingFunctionArgument(arguments)
      );
    },
    getConfig(key, scope) {
      return this._runTask(
        getConfigTask(key, asConfigScope(scope, void 0)),
        trailingFunctionArgument(arguments)
      );
    },
    listConfig(...rest) {
      return this._runTask(
        listConfigTask(asConfigScope(rest[0], void 0)),
        trailingFunctionArgument(arguments)
      );
    }
  };
}
function isDiffNameStatus(input) {
  return diffNameStatus.has(input);
}
function grepQueryBuilder(...params) {
  return new GrepQuery().param(...params);
}
function parseGrep(grep) {
  const paths = /* @__PURE__ */ new Set();
  const results = {};
  forEachLineWithContent(grep, (input) => {
    const [path2, line, preview] = input.split(NULL);
    paths.add(path2);
    (results[path2] = results[path2] || []).push({
      line: asNumber(line),
      path: path2,
      preview
    });
  });
  return {
    paths,
    results
  };
}
function grep_default() {
  return {
    grep(searchTerm) {
      const then = trailingFunctionArgument(arguments);
      const options = getTrailingOptions(arguments);
      for (const option of disallowedOptions) {
        if (options.includes(option)) {
          return this._runTask(
            configurationErrorTask(`git.grep: use of "${option}" is not supported.`),
            then
          );
        }
      }
      if (typeof searchTerm === "string") {
        searchTerm = grepQueryBuilder().param(searchTerm);
      }
      const commands = ["grep", "--null", "-n", "--full-name", ...options, ...searchTerm];
      return this._runTask(
        {
          commands,
          format: "utf-8",
          parser(stdOut) {
            return parseGrep(stdOut);
          }
        },
        then
      );
    }
  };
}
function resetTask(mode, customArgs) {
  const commands = ["reset"];
  if (isValidResetMode(mode)) {
    commands.push(`--${mode}`);
  }
  commands.push(...customArgs);
  return straightThroughStringTask(commands);
}
function getResetMode(mode) {
  if (isValidResetMode(mode)) {
    return mode;
  }
  switch (typeof mode) {
    case "string":
    case "undefined":
      return "soft";
  }
  return;
}
function isValidResetMode(mode) {
  return typeof mode === "string" && validResetModes.includes(mode);
}
function createLog() {
  return (0, import_debug.default)("simple-git");
}
function prefixedLogger(to, prefix, forward) {
  if (!prefix || !String(prefix).replace(/\s*/, "")) {
    return !forward ? to : (message, ...args) => {
      to(message, ...args);
      forward(message, ...args);
    };
  }
  return (message, ...args) => {
    to(`%s ${message}`, prefix, ...args);
    if (forward) {
      forward(message, ...args);
    }
  };
}
function childLoggerName(name, childDebugger, { namespace: parentNamespace }) {
  if (typeof name === "string") {
    return name;
  }
  const childNamespace = childDebugger && childDebugger.namespace || "";
  if (childNamespace.startsWith(parentNamespace)) {
    return childNamespace.substr(parentNamespace.length + 1);
  }
  return childNamespace || parentNamespace;
}
function createLogger(label, verbose, initialStep, infoDebugger = createLog()) {
  const labelPrefix = label && `[${label}]` || "";
  const spawned = [];
  const debugDebugger = typeof verbose === "string" ? infoDebugger.extend(verbose) : verbose;
  const key = childLoggerName(filterType(verbose, filterString), debugDebugger, infoDebugger);
  return step(initialStep);
  function sibling(name, initial) {
    return append(
      spawned,
      createLogger(label, key.replace(/^[^:]+/, name), initial, infoDebugger)
    );
  }
  function step(phase) {
    const stepPrefix = phase && `[${phase}]` || "";
    const debug2 = debugDebugger && prefixedLogger(debugDebugger, stepPrefix) || NOOP;
    const info = prefixedLogger(infoDebugger, `${labelPrefix} ${stepPrefix}`, debug2);
    return Object.assign(debugDebugger ? debug2 : info, {
      label,
      sibling,
      info,
      step
    });
  }
}
function pluginContext(task, commands) {
  return {
    method: first(task.commands) || "",
    commands
  };
}
function onErrorReceived(target, logger) {
  return (err) => {
    logger(`[ERROR] child process exception %o`, err);
    target.push(Buffer.from(String(err.stack), "ascii"));
  };
}
function onDataReceived(target, name, logger, output) {
  return (buffer) => {
    logger(`%s received %L bytes`, name, buffer);
    output(`%B`, buffer);
    target.push(buffer);
  };
}
function taskCallback(task, response, callback = NOOP) {
  const onSuccess = (data) => {
    callback(null, data);
  };
  const onError2 = (err) => {
    if (err?.task === task) {
      callback(
        err instanceof GitResponseError ? addDeprecationNoticeToError(err) : err,
        void 0
      );
    }
  };
  response.then(onSuccess, onError2);
}
function addDeprecationNoticeToError(err) {
  let log = (name) => {
    console.warn(
      `simple-git deprecation notice: accessing GitResponseError.${name} should be GitResponseError.git.${name}, this will no longer be available in version 3`
    );
    log = NOOP;
  };
  return Object.create(err, Object.getOwnPropertyNames(err.git).reduce(descriptorReducer, {}));
  function descriptorReducer(all, name) {
    if (name in err) {
      return all;
    }
    all[name] = {
      enumerable: false,
      configurable: false,
      get() {
        log(name);
        return err.git[name];
      }
    };
    return all;
  }
}
function changeWorkingDirectoryTask(directory, root) {
  return adhocExecTask((instance) => {
    if (!folderExists(directory)) {
      throw new Error(`Git.cwd: cannot change to non-directory "${directory}"`);
    }
    return (root || instance).cwd = directory;
  });
}
function checkoutTask(args) {
  const commands = ["checkout", ...args];
  if (commands[1] === "-b" && commands.includes("-B")) {
    commands[1] = remove(commands, "-B");
  }
  return straightThroughStringTask(commands);
}
function checkout_default() {
  return {
    checkout() {
      return this._runTask(
        checkoutTask(getTrailingOptions(arguments, 1)),
        trailingFunctionArgument(arguments)
      );
    },
    checkoutBranch(branchName, startPoint) {
      return this._runTask(
        checkoutTask(["-b", branchName, startPoint, ...getTrailingOptions(arguments)]),
        trailingFunctionArgument(arguments)
      );
    },
    checkoutLocalBranch(branchName) {
      return this._runTask(
        checkoutTask(["-b", branchName, ...getTrailingOptions(arguments)]),
        trailingFunctionArgument(arguments)
      );
    }
  };
}
function countObjectsResponse() {
  return {
    count: 0,
    garbage: 0,
    inPack: 0,
    packs: 0,
    prunePackable: 0,
    size: 0,
    sizeGarbage: 0,
    sizePack: 0
  };
}
function count_objects_default() {
  return {
    countObjects() {
      return this._runTask({
        commands: ["count-objects", "--verbose"],
        format: "utf-8",
        parser(stdOut) {
          return parseStringResponse(countObjectsResponse(), [parser2], stdOut);
        }
      });
    }
  };
}
function parseCommitResult(stdOut) {
  const result = {
    author: null,
    branch: "",
    commit: "",
    root: false,
    summary: {
      changes: 0,
      insertions: 0,
      deletions: 0
    }
  };
  return parseStringResponse(result, parsers, stdOut);
}
function commitTask(message, files, customArgs) {
  const commands = [
    "-c",
    "core.abbrev=40",
    "commit",
    ...prefixedArray(message, "-m"),
    ...files,
    ...customArgs
  ];
  return {
    commands,
    format: "utf-8",
    parser: parseCommitResult
  };
}
function commit_default() {
  return {
    commit(message, ...rest) {
      const next = trailingFunctionArgument(arguments);
      const task = rejectDeprecatedSignatures(message) || commitTask(
        asArray(message),
        asArray(filterType(rest[0], filterStringOrStringArray, [])),
        [
          ...asStringArray(filterType(rest[1], filterArray, [])),
          ...getTrailingOptions(arguments, 0, true)
        ]
      );
      return this._runTask(task, next);
    }
  };
  function rejectDeprecatedSignatures(message) {
    return !filterStringOrStringArray(message) && configurationErrorTask(
      `git.commit: requires the commit message to be supplied as a string/string[]`
    );
  }
}
function first_commit_default() {
  return {
    firstCommit() {
      return this._runTask(
        straightThroughStringTask(["rev-list", "--max-parents=0", "HEAD"], true),
        trailingFunctionArgument(arguments)
      );
    }
  };
}
function hashObjectTask(filePath, write) {
  const commands = ["hash-object", filePath];
  if (write) {
    commands.push("-w");
  }
  return straightThroughStringTask(commands, true);
}
function parseInit(bare, path2, text) {
  const response = String(text).trim();
  let result;
  if (result = initResponseRegex.exec(response)) {
    return new InitSummary(bare, path2, false, result[1]);
  }
  if (result = reInitResponseRegex.exec(response)) {
    return new InitSummary(bare, path2, true, result[1]);
  }
  let gitDir = "";
  const tokens = response.split(" ");
  while (tokens.length) {
    const token = tokens.shift();
    if (token === "in") {
      gitDir = tokens.join(" ");
      break;
    }
  }
  return new InitSummary(bare, path2, /^re/i.test(response), gitDir);
}
function hasBareCommand(command) {
  return command.includes(bareCommand);
}
function initTask(bare = false, path2, customArgs) {
  const commands = ["init", ...customArgs];
  if (bare && !hasBareCommand(commands)) {
    commands.splice(1, 0, bareCommand);
  }
  return {
    commands,
    format: "utf-8",
    parser(text) {
      return parseInit(commands.includes("--bare"), path2, text);
    }
  };
}
function logFormatFromCommand(customArgs) {
  for (let i2 = 0; i2 < customArgs.length; i2++) {
    const format = logFormatRegex.exec(customArgs[i2]);
    if (format) {
      return `--${format[1]}`;
    }
  }
  return "";
}
function isLogFormat(customArg) {
  return logFormatRegex.test(customArg);
}
function getDiffParser(format = "") {
  const parser4 = diffSummaryParsers[format];
  return (stdOut) => parseStringResponse(new DiffSummary(), parser4, stdOut, false);
}
function lineBuilder(tokens, fields) {
  return fields.reduce(
    (line, field, index) => {
      line[field] = tokens[index] || "";
      return line;
    },
    /* @__PURE__ */ Object.create({ diff: null })
  );
}
function createListLogSummaryParser(splitter = SPLITTER, fields = defaultFieldNames, logFormat = "") {
  const parseDiffResult = getDiffParser(logFormat);
  return function(stdOut) {
    const all = toLinesWithContent(
      stdOut.trim(),
      false,
      START_BOUNDARY
    ).map(function(item) {
      const lineDetail = item.split(COMMIT_BOUNDARY);
      const listLogLine = lineBuilder(lineDetail[0].split(splitter), fields);
      if (lineDetail.length > 1 && !!lineDetail[1].trim()) {
        listLogLine.diff = parseDiffResult(lineDetail[1]);
      }
      return listLogLine;
    });
    return {
      all,
      latest: all.length && all[0] || null,
      total: all.length
    };
  };
}
function diffSummaryTask(customArgs) {
  let logFormat = logFormatFromCommand(customArgs);
  const commands = ["diff"];
  if (logFormat === "") {
    logFormat = "--stat";
    commands.push("--stat=4096");
  }
  commands.push(...customArgs);
  return validateLogFormatConfig(commands) || {
    commands,
    format: "utf-8",
    parser: getDiffParser(logFormat)
  };
}
function validateLogFormatConfig(customArgs) {
  const flags = customArgs.filter(isLogFormat);
  if (flags.length > 1) {
    return configurationErrorTask(
      `Summary flags are mutually exclusive - pick one of ${flags.join(",")}`
    );
  }
  if (flags.length && customArgs.includes("-z")) {
    return configurationErrorTask(
      `Summary flag ${flags} parsing is not compatible with null termination option '-z'`
    );
  }
}
function prettyFormat(format, splitter) {
  const fields = [];
  const formatStr = [];
  Object.keys(format).forEach((field) => {
    fields.push(field);
    formatStr.push(String(format[field]));
  });
  return [fields, formatStr.join(splitter)];
}
function userOptions(input) {
  return Object.keys(input).reduce((out, key) => {
    if (!(key in excludeOptions)) {
      out[key] = input[key];
    }
    return out;
  }, {});
}
function parseLogOptions(opt = {}, customArgs = []) {
  const splitter = filterType(opt.splitter, filterString, SPLITTER);
  const format = filterPlainObject(opt.format) ? opt.format : {
    hash: "%H",
    date: opt.strictDate === false ? "%ai" : "%aI",
    message: "%s",
    refs: "%D",
    body: opt.multiLine ? "%B" : "%b",
    author_name: opt.mailMap !== false ? "%aN" : "%an",
    author_email: opt.mailMap !== false ? "%aE" : "%ae"
  };
  const [fields, formatStr] = prettyFormat(format, splitter);
  const suffix = [];
  const command = [
    `--pretty=format:${START_BOUNDARY}${formatStr}${COMMIT_BOUNDARY}`,
    ...customArgs
  ];
  const maxCount = opt.n || opt["max-count"] || opt.maxCount;
  if (maxCount) {
    command.push(`--max-count=${maxCount}`);
  }
  if (opt.from || opt.to) {
    const rangeOperator = opt.symmetric !== false ? "..." : "..";
    suffix.push(`${opt.from || ""}${rangeOperator}${opt.to || ""}`);
  }
  if (filterString(opt.file)) {
    command.push("--follow", c(opt.file));
  }
  appendTaskOptions(userOptions(opt), command);
  return {
    fields,
    splitter,
    commands: [...command, ...suffix]
  };
}
function logTask(splitter, fields, customArgs) {
  const parser4 = createListLogSummaryParser(splitter, fields, logFormatFromCommand(customArgs));
  return {
    commands: ["log", ...customArgs],
    format: "utf-8",
    parser: parser4
  };
}
function log_default() {
  return {
    log(...rest) {
      const next = trailingFunctionArgument(arguments);
      const options = parseLogOptions(
        trailingOptionsArgument(arguments),
        asStringArray(filterType(arguments[0], filterArray, []))
      );
      const task = rejectDeprecatedSignatures(...rest) || validateLogFormatConfig(options.commands) || createLogTask(options);
      return this._runTask(task, next);
    }
  };
  function createLogTask(options) {
    return logTask(options.splitter, options.fields, options.commands);
  }
  function rejectDeprecatedSignatures(from, to) {
    return filterString(from) && filterString(to) && configurationErrorTask(
      `git.log(string, string) should be replaced with git.log({ from: string, to: string })`
    );
  }
}
function objectEnumerationResult(remoteMessages) {
  return remoteMessages.objects = remoteMessages.objects || {
    compressing: 0,
    counting: 0,
    enumerating: 0,
    packReused: 0,
    reused: { count: 0, delta: 0 },
    total: { count: 0, delta: 0 }
  };
}
function asObjectCount(source) {
  const count = /^\s*(\d+)/.exec(source);
  const delta = /delta (\d+)/i.exec(source);
  return {
    count: asNumber(count && count[1] || "0"),
    delta: asNumber(delta && delta[1] || "0")
  };
}
function parseRemoteMessages(_stdOut, stdErr) {
  return parseStringResponse({ remoteMessages: new RemoteMessageSummary() }, parsers2, stdErr);
}
function parsePullErrorResult(stdOut, stdErr) {
  const pullError = parseStringResponse(new PullFailedSummary(), errorParsers, [stdOut, stdErr]);
  return pullError.message && pullError;
}
function mergeTask(customArgs) {
  if (!customArgs.length) {
    return configurationErrorTask("Git.merge requires at least one option");
  }
  return {
    commands: ["merge", ...customArgs],
    format: "utf-8",
    parser(stdOut, stdErr) {
      const merge = parseMergeResult(stdOut, stdErr);
      if (merge.failed) {
        throw new GitResponseError(merge);
      }
      return merge;
    }
  };
}
function pushResultPushedItem(local, remote, status) {
  const deleted = status.includes("deleted");
  const tag = status.includes("tag") || /^refs\/tags/.test(local);
  const alreadyUpdated = !status.includes("new");
  return {
    deleted,
    tag,
    branch: !tag,
    new: !alreadyUpdated,
    alreadyUpdated,
    local,
    remote
  };
}
function pushTagsTask(ref = {}, customArgs) {
  append(customArgs, "--tags");
  return pushTask(ref, customArgs);
}
function pushTask(ref = {}, customArgs) {
  const commands = ["push", ...customArgs];
  if (ref.branch) {
    commands.splice(1, 0, ref.branch);
  }
  if (ref.remote) {
    commands.splice(1, 0, ref.remote);
  }
  remove(commands, "-v");
  append(commands, "--verbose");
  append(commands, "--porcelain");
  return {
    commands,
    format: "utf-8",
    parser: parsePushResult
  };
}
function show_default() {
  return {
    showBuffer() {
      const commands = ["show", ...getTrailingOptions(arguments, 1)];
      if (!commands.includes("--binary")) {
        commands.splice(1, 0, "--binary");
      }
      return this._runTask(
        straightThroughBufferTask(commands),
        trailingFunctionArgument(arguments)
      );
    },
    show() {
      const commands = ["show", ...getTrailingOptions(arguments, 1)];
      return this._runTask(
        straightThroughStringTask(commands),
        trailingFunctionArgument(arguments)
      );
    }
  };
}
function renamedFile(line) {
  const [to, from] = line.split(NULL);
  return {
    from: from || to,
    to
  };
}
function parser3(indexX, indexY, handler) {
  return [`${indexX}${indexY}`, handler];
}
function conflicts(indexX, ...indexY) {
  return indexY.map((y2) => parser3(indexX, y2, (result, file) => result.conflicted.push(file)));
}
function splitLine(result, lineStr) {
  const trimmed2 = lineStr.trim();
  switch (" ") {
    case trimmed2.charAt(2):
      return data(trimmed2.charAt(0), trimmed2.charAt(1), trimmed2.slice(3));
    case trimmed2.charAt(1):
      return data(" ", trimmed2.charAt(0), trimmed2.slice(2));
    default:
      return;
  }
  function data(index, workingDir, path2) {
    const raw = `${index}${workingDir}`;
    const handler = parsers6.get(raw);
    if (handler) {
      handler(result, path2);
    }
    if (raw !== "##" && raw !== "!!") {
      result.files.push(new FileStatusSummary(path2, index, workingDir));
    }
  }
}
function statusTask(customArgs) {
  const commands = [
    "status",
    "--porcelain",
    "-b",
    "-u",
    "--null",
    ...customArgs.filter((arg) => !ignoredOptions.includes(arg))
  ];
  return {
    format: "utf-8",
    commands,
    parser(text) {
      return parseStatusSummary(text);
    }
  };
}
function versionResponse(major = 0, minor = 0, patch = 0, agent = "", installed = true) {
  return Object.defineProperty(
    {
      major,
      minor,
      patch,
      agent,
      installed
    },
    "toString",
    {
      value() {
        return `${this.major}.${this.minor}.${this.patch}`;
      },
      configurable: false,
      enumerable: false
    }
  );
}
function notInstalledResponse() {
  return versionResponse(0, 0, 0, "", false);
}
function version_default() {
  return {
    version() {
      return this._runTask({
        commands: ["--version"],
        format: "utf-8",
        parser: versionParser,
        onError(result, error, done, fail) {
          if (result.exitCode === -2) {
            return done(Buffer.from(NOT_INSTALLED));
          }
          fail(error);
        }
      });
    }
  };
}
function versionParser(stdOut) {
  if (stdOut === NOT_INSTALLED) {
    return notInstalledResponse();
  }
  return parseStringResponse(versionResponse(0, 0, 0, stdOut), parsers7, stdOut);
}
function createCloneTask(api, task, repoPath, ...args) {
  if (!filterString(repoPath)) {
    return configurationErrorTask(`git.${api}() requires a string 'repoPath'`);
  }
  return task(repoPath, filterType(args[0], filterString), getTrailingOptions(arguments));
}
function clone_default() {
  return {
    clone(repo, ...rest) {
      return this._runTask(
        createCloneTask("clone", cloneTask, filterType(repo, filterString), ...rest),
        trailingFunctionArgument(arguments)
      );
    },
    mirror(repo, ...rest) {
      return this._runTask(
        createCloneTask("mirror", cloneMirrorTask, filterType(repo, filterString), ...rest),
        trailingFunctionArgument(arguments)
      );
    }
  };
}
function applyPatchTask(patches, customArgs) {
  return straightThroughStringTask(["apply", ...customArgs, ...patches]);
}
function branchDeletionSuccess(branch, hash) {
  return {
    branch,
    hash,
    success: true
  };
}
function branchDeletionFailure(branch) {
  return {
    branch,
    hash: null,
    success: false
  };
}
function hasBranchDeletionError(data, processExitCode) {
  return processExitCode === 1 && deleteErrorRegex.test(data);
}
function branchStatus(input) {
  return input ? input.charAt(0) : "";
}
function parseBranchSummary(stdOut, currentOnly = false) {
  return parseStringResponse(
    new BranchSummaryResult(),
    currentOnly ? [currentBranchParser] : parsers9,
    stdOut
  );
}
function containsDeleteBranchCommand(commands) {
  const deleteCommands = ["-d", "-D", "--delete"];
  return commands.some((command) => deleteCommands.includes(command));
}
function branchTask(customArgs) {
  const isDelete = containsDeleteBranchCommand(customArgs);
  const isCurrentOnly = customArgs.includes("--show-current");
  const commands = ["branch", ...customArgs];
  if (commands.length === 1) {
    commands.push("-a");
  }
  if (!commands.includes("-v")) {
    commands.splice(1, 0, "-v");
  }
  return {
    format: "utf-8",
    commands,
    parser(stdOut, stdErr) {
      if (isDelete) {
        return parseBranchDeletions(stdOut, stdErr).all[0];
      }
      return parseBranchSummary(stdOut, isCurrentOnly);
    }
  };
}
function branchLocalTask() {
  return {
    format: "utf-8",
    commands: ["branch", "-v"],
    parser(stdOut) {
      return parseBranchSummary(stdOut);
    }
  };
}
function deleteBranchesTask(branches, forceDelete = false) {
  return {
    format: "utf-8",
    commands: ["branch", "-v", forceDelete ? "-D" : "-d", ...branches],
    parser(stdOut, stdErr) {
      return parseBranchDeletions(stdOut, stdErr);
    },
    onError({ exitCode, stdOut }, error, done, fail) {
      if (!hasBranchDeletionError(String(error), exitCode)) {
        return fail(error);
      }
      done(stdOut);
    }
  };
}
function deleteBranchTask(branch, forceDelete = false) {
  const task = {
    format: "utf-8",
    commands: ["branch", "-v", forceDelete ? "-D" : "-d", branch],
    parser(stdOut, stdErr) {
      return parseBranchDeletions(stdOut, stdErr).branches[branch];
    },
    onError({ exitCode, stdErr, stdOut }, error, _2, fail) {
      if (!hasBranchDeletionError(String(error), exitCode)) {
        return fail(error);
      }
      throw new GitResponseError(
        task.parser(bufferToString(stdOut), bufferToString(stdErr)),
        String(error)
      );
    }
  };
  return task;
}
function toPath(input) {
  const path2 = input.trim().replace(/^["']|["']$/g, "");
  return path2 && normalize(path2);
}
function checkIgnoreTask(paths) {
  return {
    commands: ["check-ignore", ...paths],
    format: "utf-8",
    parser: parseCheckIgnore
  };
}
function parseFetchResult(stdOut, stdErr) {
  const result = {
    raw: stdOut,
    remote: null,
    branches: [],
    tags: [],
    updated: [],
    deleted: []
  };
  return parseStringResponse(result, parsers10, [stdOut, stdErr]);
}
function disallowedCommand(command) {
  return /^--upload-pack(=|$)/.test(command);
}
function fetchTask(remote, branch, customArgs) {
  const commands = ["fetch", ...customArgs];
  if (remote && branch) {
    commands.push(remote, branch);
  }
  const banned = commands.find(disallowedCommand);
  if (banned) {
    return configurationErrorTask(`git.fetch: potential exploit argument blocked.`);
  }
  return {
    commands,
    format: "utf-8",
    parser: parseFetchResult
  };
}
function parseMoveResult(stdOut) {
  return parseStringResponse({ moves: [] }, parsers11, stdOut);
}
function moveTask(from, to) {
  return {
    commands: ["mv", "-v", ...asArray(from), to],
    format: "utf-8",
    parser: parseMoveResult
  };
}
function pullTask(remote, branch, customArgs) {
  const commands = ["pull", ...customArgs];
  if (remote && branch) {
    commands.splice(1, 0, remote, branch);
  }
  return {
    commands,
    format: "utf-8",
    parser(stdOut, stdErr) {
      return parsePullResult(stdOut, stdErr);
    },
    onError(result, _error, _done, fail) {
      const pullError = parsePullErrorResult(
        bufferToString(result.stdOut),
        bufferToString(result.stdErr)
      );
      if (pullError) {
        return fail(new GitResponseError(pullError));
      }
      fail(_error);
    }
  };
}
function parseGetRemotes(text) {
  const remotes = {};
  forEach(text, ([name]) => remotes[name] = { name });
  return Object.values(remotes);
}
function parseGetRemotesVerbose(text) {
  const remotes = {};
  forEach(text, ([name, url, purpose]) => {
    if (!Object.hasOwn(remotes, name)) {
      remotes[name] = {
        name,
        refs: { fetch: "", push: "" }
      };
    }
    if (purpose && url) {
      remotes[name].refs[purpose.replace(/[^a-z]/g, "")] = url;
    }
  });
  return Object.values(remotes);
}
function forEach(text, handler) {
  forEachLineWithContent(text, (line) => handler(line.split(/\s+/)));
}
function addRemoteTask(remoteName, remoteRepo, customArgs) {
  return straightThroughStringTask(["remote", "add", ...customArgs, remoteName, remoteRepo]);
}
function getRemotesTask(verbose) {
  const commands = ["remote"];
  if (verbose) {
    commands.push("-v");
  }
  return {
    commands,
    format: "utf-8",
    parser: verbose ? parseGetRemotesVerbose : parseGetRemotes
  };
}
function listRemotesTask(customArgs) {
  const commands = [...customArgs];
  if (commands[0] !== "ls-remote") {
    commands.unshift("ls-remote");
  }
  return straightThroughStringTask(commands);
}
function remoteTask(customArgs) {
  const commands = [...customArgs];
  if (commands[0] !== "remote") {
    commands.unshift("remote");
  }
  return straightThroughStringTask(commands);
}
function removeRemoteTask(remoteName) {
  return straightThroughStringTask(["remote", "remove", remoteName]);
}
function stashListTask(opt = {}, customArgs) {
  const options = parseLogOptions(opt);
  const commands = ["stash", "list", ...options.commands, ...customArgs];
  const parser4 = createListLogSummaryParser(
    options.splitter,
    options.fields,
    logFormatFromCommand(commands)
  );
  return validateLogFormatConfig(commands) || {
    commands,
    format: "utf-8",
    parser: parser4
  };
}
function addSubModuleTask(repo, path2) {
  return subModuleTask(["add", repo, path2]);
}
function initSubModuleTask(customArgs) {
  return subModuleTask(["init", ...customArgs]);
}
function subModuleTask(customArgs) {
  const commands = [...customArgs];
  if (commands[0] !== "submodule") {
    commands.unshift("submodule");
  }
  return straightThroughStringTask(commands);
}
function updateSubModuleTask(customArgs) {
  return subModuleTask(["update", ...customArgs]);
}
function singleSorted(a, b2) {
  const aIsNum = Number.isNaN(a);
  const bIsNum = Number.isNaN(b2);
  if (aIsNum !== bIsNum) {
    return aIsNum ? 1 : -1;
  }
  return aIsNum ? sorted(a, b2) : 0;
}
function sorted(a, b2) {
  return a === b2 ? 0 : a > b2 ? 1 : -1;
}
function trimmed(input) {
  return input.trim();
}
function toNumber(input) {
  if (typeof input === "string") {
    return parseInt(input.replace(/^\D+/g, ""), 10) || 0;
  }
  return 0;
}
function tagListTask(customArgs = []) {
  const hasCustomSort = customArgs.some((option) => /^--sort=/.test(option));
  return {
    format: "utf-8",
    commands: ["tag", "-l", ...customArgs],
    parser(text) {
      return parseTagList(text, hasCustomSort);
    }
  };
}
function addTagTask(name) {
  return {
    format: "utf-8",
    commands: ["tag", name],
    parser() {
      return { name };
    }
  };
}
function addAnnotatedTagTask(name, tagMessage) {
  return {
    format: "utf-8",
    commands: ["tag", "-a", "-m", tagMessage, name],
    parser() {
      return { name };
    }
  };
}
function abortPlugin(signal) {
  if (!signal) {
    return;
  }
  const onSpawnAfter = {
    type: "spawn.after",
    action(_data, context) {
      function kill() {
        context.kill(new GitPluginError(void 0, "abort", "Abort signal received"));
      }
      signal.addEventListener("abort", kill);
      context.spawned.on("close", () => signal.removeEventListener("abort", kill));
    }
  };
  const onSpawnBefore = {
    type: "spawn.before",
    action(_data, context) {
      if (signal.aborted) {
        context.kill(new GitPluginError(void 0, "abort", "Abort already signaled"));
      }
    }
  };
  return [onSpawnBefore, onSpawnAfter];
}
function blockUnsafeOperationsPlugin(options = {}) {
  return {
    type: "spawn.args",
    action(args, { env: env2 }) {
      for (const vulnerability of ne(args, env2)) {
        if (options[vulnerability.category] !== true) {
          throw new GitPluginError(void 0, "unsafe", vulnerability.message);
        }
      }
      return args;
    }
  };
}
function commandConfigPrefixingPlugin(configuration) {
  const prefix = prefixedArray(configuration, "-c");
  return {
    type: "spawn.args",
    action(data) {
      return [...prefix, ...data];
    }
  };
}
function completionDetectionPlugin({
  onClose = true,
  onExit = 50
} = {}) {
  function createEvents() {
    let exitCode = -1;
    const events = {
      close: (0, import_promise_deferred2.deferred)(),
      closeTimeout: (0, import_promise_deferred2.deferred)(),
      exit: (0, import_promise_deferred2.deferred)(),
      exitTimeout: (0, import_promise_deferred2.deferred)()
    };
    const result = Promise.race([
      onClose === false ? never : events.closeTimeout.promise,
      onExit === false ? never : events.exitTimeout.promise
    ]);
    configureTimeout(onClose, events.close, events.closeTimeout);
    configureTimeout(onExit, events.exit, events.exitTimeout);
    return {
      close(code) {
        exitCode = code;
        events.close.done();
      },
      exit(code) {
        exitCode = code;
        events.exit.done();
      },
      get exitCode() {
        return exitCode;
      },
      result
    };
  }
  function configureTimeout(flag, event, timeout) {
    if (flag === false) {
      return;
    }
    (flag === true ? event.promise : event.promise.then(() => delay(flag))).then(timeout.done);
  }
  return {
    type: "spawn.after",
    async action(_data, { spawned, close }) {
      const events = createEvents();
      let deferClose = true;
      let quickClose = () => void (deferClose = false);
      spawned.stdout?.on("data", quickClose);
      spawned.stderr?.on("data", quickClose);
      spawned.on("error", quickClose);
      spawned.on("close", (code) => events.close(code));
      spawned.on("exit", (code) => events.exit(code));
      try {
        await events.result;
        if (deferClose) {
          await delay(50);
        }
        close(events.exitCode);
      } catch (err) {
        close(events.exitCode, err);
      }
    }
  };
}
function isBadArgument(arg) {
  return !arg || !/^([a-z]:)?([a-z0-9/.\\_~-]+)$/i.test(arg);
}
function toBinaryConfig(input, allowUnsafe) {
  if (input.length < 1 || input.length > 2) {
    throw new GitPluginError(void 0, "binary", WRONG_NUMBER_ERR);
  }
  const isBad = input.some(isBadArgument);
  if (isBad) {
    if (allowUnsafe) {
      console.warn(WRONG_CHARS_ERR);
    } else {
      throw new GitPluginError(void 0, "binary", WRONG_CHARS_ERR);
    }
  }
  const [binary, prefix] = input;
  return {
    binary,
    prefix
  };
}
function customBinaryPlugin(plugins, input = ["git"], allowUnsafe = false) {
  let config = toBinaryConfig(asArray(input), allowUnsafe);
  plugins.on("binary", (input2) => {
    config = toBinaryConfig(asArray(input2), allowUnsafe);
  });
  plugins.append("spawn.binary", () => {
    return config.binary;
  });
  plugins.append("spawn.args", (data) => {
    return config.prefix ? [config.prefix, ...data] : data;
  });
}
function isTaskError(result) {
  return !!(result.exitCode && result.stdErr.length);
}
function getErrorMessage(result) {
  return Buffer.concat([...result.stdOut, ...result.stdErr]);
}
function errorDetectionHandler(overwrite = false, isError = isTaskError, errorMessage = getErrorMessage) {
  return (error, result) => {
    if (!overwrite && error || !isError(result)) {
      return error;
    }
    return errorMessage(result);
  };
}
function errorDetectionPlugin(config) {
  return {
    type: "task.error",
    action(data, context) {
      const error = config(data.error, {
        stdErr: context.stdErr,
        stdOut: context.stdOut,
        exitCode: context.exitCode
      });
      if (Buffer.isBuffer(error)) {
        return { error: new GitError(void 0, error.toString("utf-8")) };
      }
      return {
        error
      };
    }
  };
}
function progressMonitorPlugin(progress) {
  const progressCommand = "--progress";
  const progressMethods = ["checkout", "clone", "fetch", "pull", "push"];
  const onProgress = {
    type: "spawn.after",
    action(_data, context) {
      if (!context.commands.includes(progressCommand)) {
        return;
      }
      context.spawned.stderr?.on("data", (chunk) => {
        const message = /^([\s\S]+?):\s*(\d+)% \((\d+)\/(\d+)\)/.exec(chunk.toString("utf8"));
        if (!message) {
          return;
        }
        progress({
          method: context.method,
          stage: progressEventStage(message[1]),
          progress: asNumber(message[2]),
          processed: asNumber(message[3]),
          total: asNumber(message[4])
        });
      });
    }
  };
  const onArgs = {
    type: "spawn.args",
    action(args, context) {
      if (!progressMethods.includes(context.method)) {
        return args;
      }
      return including(args, progressCommand);
    }
  };
  return [onArgs, onProgress];
}
function progressEventStage(input) {
  return String(input.toLowerCase().split(" ", 1)) || "unknown";
}
function spawnOptionsPlugin(spawnOptions) {
  const options = pick(spawnOptions, ["uid", "gid"]);
  return {
    type: "spawn.options",
    action(data) {
      return { ...options, ...data };
    }
  };
}
function timeoutPlugin({
  block,
  stdErr = true,
  stdOut = true
}) {
  if (block > 0) {
    return {
      type: "spawn.after",
      action(_data, context) {
        let timeout;
        function wait() {
          timeout && clearTimeout(timeout);
          timeout = setTimeout(kill, block);
        }
        function stop() {
          context.spawned.stdout?.off("data", wait);
          context.spawned.stderr?.off("data", wait);
          context.spawned.off("exit", stop);
          context.spawned.off("close", stop);
          timeout && clearTimeout(timeout);
        }
        function kill() {
          stop();
          context.kill(new GitPluginError(void 0, "timeout", `block timeout reached`));
        }
        stdOut && context.spawned.stdout?.on("data", wait);
        stdErr && context.spawned.stderr?.on("data", wait);
        context.spawned.on("exit", stop);
        context.spawned.on("close", stop);
        wait();
      }
    };
  }
}
function suffixPathsPlugin() {
  return {
    type: "spawn.args",
    action(data) {
      const prefix = [];
      let suffix;
      function append2(args) {
        (suffix = suffix || []).push(...args);
      }
      for (let i2 = 0; i2 < data.length; i2++) {
        const param = data[i2];
        if (r(param)) {
          append2(o(param));
          continue;
        }
        if (param === "--") {
          append2(
            data.slice(i2 + 1).flatMap((item) => r(item) && o(item) || item)
          );
          break;
        }
        prefix.push(param);
      }
      return !suffix ? prefix : [...prefix, "--", ...suffix.map(String)];
    }
  };
}
function gitInstanceFactory(baseDir, options) {
  const plugins = new PluginStore();
  const config = createInstanceConfig(
    baseDir && (typeof baseDir === "string" ? { baseDir } : baseDir) || {},
    options
  );
  if (!folderExists(config.baseDir)) {
    throw new GitConstructError(
      config,
      `Cannot use simple-git on a directory that does not exist`
    );
  }
  if (Array.isArray(config.config)) {
    plugins.add(commandConfigPrefixingPlugin(config.config));
  }
  plugins.add(blockUnsafeOperationsPlugin(config.unsafe));
  plugins.add(completionDetectionPlugin(config.completion));
  config.abort && plugins.add(abortPlugin(config.abort));
  config.progress && plugins.add(progressMonitorPlugin(config.progress));
  config.timeout && plugins.add(timeoutPlugin(config.timeout));
  config.spawnOptions && plugins.add(spawnOptionsPlugin(config.spawnOptions));
  plugins.add(suffixPathsPlugin());
  plugins.add(errorDetectionPlugin(errorDetectionHandler(true)));
  config.errors && plugins.add(errorDetectionPlugin(config.errors));
  customBinaryPlugin(plugins, config.binary, config.unsafe?.allowUnsafeCustomBinary);
  return new Git(config, plugins);
}
function gitP(...args) {
  let git;
  let chain = Promise.resolve();
  try {
    git = gitInstanceFactory(...args);
  } catch (e) {
    chain = Promise.reject(e);
  }
  function builderReturn() {
    return promiseApi;
  }
  function chainReturn() {
    return chain;
  }
  const promiseApi = [...functionNamesBuilderApi, ...functionNamesPromiseApi].reduce(
    (api, name) => {
      const isAsync = functionNamesPromiseApi.includes(name);
      const valid = isAsync ? asyncWrapper(name, git) : syncWrapper(name, git, api);
      const alternative = isAsync ? chainReturn : builderReturn;
      Object.defineProperty(api, name, {
        enumerable: false,
        configurable: false,
        value: git ? valid : alternative
      });
      return api;
    },
    {}
  );
  return promiseApi;
  function asyncWrapper(fn, git2) {
    return function(...args2) {
      if (typeof args2[args2.length] === "function") {
        throw new TypeError(
          "Promise interface requires that handlers are not supplied inline, trailing function not allowed in call to " + fn
        );
      }
      return chain.then(function() {
        return new Promise(function(resolve, reject) {
          const callback = (err, result) => {
            if (err) {
              return reject(toError(err));
            }
            resolve(result);
          };
          args2.push(callback);
          git2[fn].apply(git2, args2);
        });
      });
    };
  }
  function syncWrapper(fn, git2, api) {
    return (...args2) => {
      git2[fn](...args2);
      return api;
    };
  }
}
function toError(error) {
  if (error instanceof Error) {
    return error;
  }
  if (typeof error === "string") {
    return new Error(error);
  }
  return new GitResponseError(error);
}
var import_file_exists, import_debug, import_promise_deferred, import_promise_deferred2, __defProp2, __getOwnPropDesc2, __getOwnPropNames2, __hasOwnProp2, __esm2, __commonJS2, __export2, __copyProps2, __toCommonJS, GitError, init_git_error, GitResponseError, init_git_response_error, TaskConfigurationError, init_task_configuration_error, NULL, NOOP, objectToString, init_util, filterArray, filterNumber, filterString, filterStringOrStringArray, filterHasLength, init_argument_filters, ExitCodes, init_exit_codes, GitOutputStreams, init_git_output_streams, LineParser, RemoteLineParser, init_line_parser, defaultOptions, init_simple_git_options, init_task_options, init_task_parser, utils_exports, init_utils, check_is_repo_exports, CheckRepoActions, onError, parser, init_check_is_repo, CleanResponse, removalRegexp, dryRunRemovalRegexp, isFolderRegexp, init_CleanSummary, task_exports, EMPTY_COMMANDS, init_task, clean_exports, CONFIG_ERROR_INTERACTIVE_MODE, CONFIG_ERROR_MODE_REQUIRED, CONFIG_ERROR_UNKNOWN_OPTION, CleanOptions, CleanOptionValues, init_clean, ConfigList, init_ConfigList, GitConfigScope, init_config, DiffNameStatus, diffNameStatus, init_diff_name_status, disallowedOptions, Query, _a, GrepQuery, init_grep, reset_exports, ResetMode, validResetModes, init_reset, init_git_logger, TasksPendingQueue, init_tasks_pending_queue, GitExecutorChain, init_git_executor_chain, git_executor_exports, GitExecutor, init_git_executor, init_task_callback, init_change_working_directory, init_checkout, parser2, init_count_objects, parsers, init_parse_commit, init_commit, init_first_commit, init_hash_object, InitSummary, initResponseRegex, reInitResponseRegex, init_InitSummary, bareCommand, init_init, logFormatRegex, init_log_format, DiffSummary, init_DiffSummary, statParser, numStatParser, nameOnlyParser, nameStatusParser, diffSummaryParsers, init_parse_diff_summary, START_BOUNDARY, COMMIT_BOUNDARY, SPLITTER, defaultFieldNames, init_parse_list_log_summary, diff_exports, init_diff, excludeOptions, init_log, MergeSummaryConflict, MergeSummaryDetail, init_MergeSummary, PullSummary, PullFailedSummary, init_PullSummary, remoteMessagesObjectParsers, init_parse_remote_objects, parsers2, RemoteMessageSummary, init_parse_remote_messages, FILE_UPDATE_REGEX, SUMMARY_REGEX, ACTION_REGEX, parsers3, errorParsers, parsePullDetail, parsePullResult, init_parse_pull, parsers4, parseMergeResult, parseMergeDetail, init_parse_merge, init_merge, parsers5, parsePushResult, parsePushDetail, init_parse_push, push_exports, init_push, init_show, fromPathRegex, FileStatusSummary, init_FileStatusSummary, StatusSummary, parsers6, parseStatusSummary, init_StatusSummary, ignoredOptions, init_status, NOT_INSTALLED, parsers7, init_version, cloneTask, cloneMirrorTask, init_clone, simple_git_api_exports, SimpleGitApi, init_simple_git_api, scheduler_exports, createScheduledTask, Scheduler, init_scheduler, apply_patch_exports, init_apply_patch, BranchDeletionBatch, init_BranchDeleteSummary, deleteSuccessRegex, deleteErrorRegex, parsers8, parseBranchDeletions, init_parse_branch_delete, BranchSummaryResult, init_BranchSummary, parsers9, currentBranchParser, init_parse_branch, branch_exports, init_branch, parseCheckIgnore, init_CheckIgnore, check_ignore_exports, init_check_ignore, parsers10, init_parse_fetch, fetch_exports, init_fetch, parsers11, init_parse_move, move_exports, init_move, pull_exports, init_pull, init_GetRemoteSummary, remote_exports, init_remote, stash_list_exports, init_stash_list, sub_module_exports, init_sub_module, TagList, parseTagList, init_TagList, tag_exports, init_tag, require_git, GitConstructError, GitPluginError, never, WRONG_NUMBER_ERR, WRONG_CHARS_ERR, PluginStore, Git, functionNamesBuilderApi, functionNamesPromiseApi, simpleGit, esm_default;
var init_esm = __esm({
  "node_modules/simple-git/dist/esm/index.js"() {
    import_file_exists = __toESM(require_dist(), 1);
    init_dist();
    init_dist();
    import_debug = __toESM(require_src(), 1);
    init_dist();
    init_dist();
    import_promise_deferred = __toESM(require_dist2(), 1);
    init_dist();
    init_dist2();
    import_promise_deferred2 = __toESM(require_dist2(), 1);
    init_dist();
    __defProp2 = Object.defineProperty;
    __getOwnPropDesc2 = Object.getOwnPropertyDescriptor;
    __getOwnPropNames2 = Object.getOwnPropertyNames;
    __hasOwnProp2 = Object.prototype.hasOwnProperty;
    __esm2 = (fn, res) => function __init() {
      return fn && (res = (0, fn[__getOwnPropNames2(fn)[0]])(fn = 0)), res;
    };
    __commonJS2 = (cb, mod) => function __require2() {
      return mod || (0, cb[__getOwnPropNames2(cb)[0]])((mod = { exports: {} }).exports, mod), mod.exports;
    };
    __export2 = (target, all) => {
      for (var name in all)
        __defProp2(target, name, { get: all[name], enumerable: true });
    };
    __copyProps2 = (to, from, except, desc) => {
      if (from && typeof from === "object" || typeof from === "function") {
        for (let key of __getOwnPropNames2(from))
          if (!__hasOwnProp2.call(to, key) && key !== except)
            __defProp2(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc2(from, key)) || desc.enumerable });
      }
      return to;
    };
    __toCommonJS = (mod) => __copyProps2(__defProp2({}, "__esModule", { value: true }), mod);
    init_git_error = __esm2({
      "src/lib/errors/git-error.ts"() {
        "use strict";
        GitError = class extends Error {
          constructor(task, message) {
            super(message);
            this.task = task;
            Object.setPrototypeOf(this, new.target.prototype);
          }
        };
      }
    });
    init_git_response_error = __esm2({
      "src/lib/errors/git-response-error.ts"() {
        "use strict";
        init_git_error();
        GitResponseError = class extends GitError {
          constructor(git, message) {
            super(void 0, message || String(git));
            this.git = git;
          }
        };
      }
    });
    init_task_configuration_error = __esm2({
      "src/lib/errors/task-configuration-error.ts"() {
        "use strict";
        init_git_error();
        TaskConfigurationError = class extends GitError {
          constructor(message) {
            super(void 0, message);
          }
        };
      }
    });
    init_util = __esm2({
      "src/lib/utils/util.ts"() {
        "use strict";
        init_argument_filters();
        NULL = "\0";
        NOOP = () => {
        };
        objectToString = Object.prototype.toString.call.bind(Object.prototype.toString);
      }
    });
    init_argument_filters = __esm2({
      "src/lib/utils/argument-filters.ts"() {
        "use strict";
        init_util();
        filterArray = (input) => {
          return Array.isArray(input);
        };
        filterNumber = (input) => {
          return typeof input === "number";
        };
        filterString = (input) => {
          return typeof input === "string" || r(input);
        };
        filterStringOrStringArray = (input) => {
          return filterString(input) || Array.isArray(input) && input.every(filterString);
        };
        filterHasLength = (input) => {
          if (input == null || "number|boolean|function".includes(typeof input)) {
            return false;
          }
          return typeof input.length === "number";
        };
      }
    });
    init_exit_codes = __esm2({
      "src/lib/utils/exit-codes.ts"() {
        "use strict";
        ExitCodes = /* @__PURE__ */ ((ExitCodes2) => {
          ExitCodes2[ExitCodes2["SUCCESS"] = 0] = "SUCCESS";
          ExitCodes2[ExitCodes2["ERROR"] = 1] = "ERROR";
          ExitCodes2[ExitCodes2["NOT_FOUND"] = -2] = "NOT_FOUND";
          ExitCodes2[ExitCodes2["UNCLEAN"] = 128] = "UNCLEAN";
          return ExitCodes2;
        })(ExitCodes || {});
      }
    });
    init_git_output_streams = __esm2({
      "src/lib/utils/git-output-streams.ts"() {
        "use strict";
        GitOutputStreams = class _GitOutputStreams {
          constructor(stdOut, stdErr) {
            this.stdOut = stdOut;
            this.stdErr = stdErr;
          }
          asStrings() {
            return new _GitOutputStreams(this.stdOut.toString("utf8"), this.stdErr.toString("utf8"));
          }
        };
      }
    });
    init_line_parser = __esm2({
      "src/lib/utils/line-parser.ts"() {
        "use strict";
        LineParser = class {
          constructor(regExp, useMatches) {
            this.matches = [];
            this.useMatches = useMatchesDefault;
            this.parse = (line, target) => {
              this.resetMatches();
              if (!this._regExp.every((reg, index) => this.addMatch(reg, index, line(index)))) {
                return false;
              }
              return this.useMatches(target, this.prepareMatches()) !== false;
            };
            this._regExp = Array.isArray(regExp) ? regExp : [regExp];
            if (useMatches) {
              this.useMatches = useMatches;
            }
          }
          resetMatches() {
            this.matches.length = 0;
          }
          prepareMatches() {
            return this.matches;
          }
          addMatch(reg, index, line) {
            const matched = line && reg.exec(line);
            if (matched) {
              this.pushMatch(index, matched);
            }
            return !!matched;
          }
          pushMatch(_index, matched) {
            this.matches.push(...matched.slice(1));
          }
        };
        RemoteLineParser = class extends LineParser {
          addMatch(reg, index, line) {
            return /^remote:\s/.test(String(line)) && super.addMatch(reg, index, line);
          }
          pushMatch(index, matched) {
            if (index > 0 || matched.length > 1) {
              super.pushMatch(index, matched);
            }
          }
        };
      }
    });
    init_simple_git_options = __esm2({
      "src/lib/utils/simple-git-options.ts"() {
        "use strict";
        defaultOptions = {
          binary: "git",
          maxConcurrentProcesses: 5,
          config: [],
          trimmed: false
        };
      }
    });
    init_task_options = __esm2({
      "src/lib/utils/task-options.ts"() {
        "use strict";
        init_argument_filters();
        init_util();
      }
    });
    init_task_parser = __esm2({
      "src/lib/utils/task-parser.ts"() {
        "use strict";
        init_util();
      }
    });
    utils_exports = {};
    __export2(utils_exports, {
      ExitCodes: () => ExitCodes,
      GitOutputStreams: () => GitOutputStreams,
      LineParser: () => LineParser,
      NOOP: () => NOOP,
      NULL: () => NULL,
      RemoteLineParser: () => RemoteLineParser,
      append: () => append,
      appendTaskOptions: () => appendTaskOptions,
      asArray: () => asArray,
      asCamelCase: () => asCamelCase,
      asFunction: () => asFunction,
      asNumber: () => asNumber,
      asStringArray: () => asStringArray,
      bufferToString: () => bufferToString,
      callTaskParser: () => callTaskParser,
      createInstanceConfig: () => createInstanceConfig,
      delay: () => delay,
      filterArray: () => filterArray,
      filterFunction: () => filterFunction,
      filterHasLength: () => filterHasLength,
      filterNumber: () => filterNumber,
      filterPlainObject: () => filterPlainObject,
      filterPrimitives: () => filterPrimitives,
      filterString: () => filterString,
      filterStringOrStringArray: () => filterStringOrStringArray,
      filterType: () => filterType,
      first: () => first,
      folderExists: () => folderExists,
      forEachLineWithContent: () => forEachLineWithContent,
      getTrailingOptions: () => getTrailingOptions,
      including: () => including,
      isUserFunction: () => isUserFunction,
      last: () => last,
      objectToString: () => objectToString,
      orVoid: () => orVoid,
      parseStringResponse: () => parseStringResponse,
      pick: () => pick,
      prefixedArray: () => prefixedArray,
      remove: () => remove,
      splitOn: () => splitOn,
      toLinesWithContent: () => toLinesWithContent,
      trailingFunctionArgument: () => trailingFunctionArgument,
      trailingOptionsArgument: () => trailingOptionsArgument
    });
    init_utils = __esm2({
      "src/lib/utils/index.ts"() {
        "use strict";
        init_argument_filters();
        init_exit_codes();
        init_git_output_streams();
        init_line_parser();
        init_simple_git_options();
        init_task_options();
        init_task_parser();
        init_util();
      }
    });
    check_is_repo_exports = {};
    __export2(check_is_repo_exports, {
      CheckRepoActions: () => CheckRepoActions,
      checkIsBareRepoTask: () => checkIsBareRepoTask,
      checkIsRepoRootTask: () => checkIsRepoRootTask,
      checkIsRepoTask: () => checkIsRepoTask
    });
    init_check_is_repo = __esm2({
      "src/lib/tasks/check-is-repo.ts"() {
        "use strict";
        init_utils();
        CheckRepoActions = /* @__PURE__ */ ((CheckRepoActions2) => {
          CheckRepoActions2["BARE"] = "bare";
          CheckRepoActions2["IN_TREE"] = "tree";
          CheckRepoActions2["IS_REPO_ROOT"] = "root";
          return CheckRepoActions2;
        })(CheckRepoActions || {});
        onError = ({ exitCode }, error, done, fail) => {
          if (exitCode === 128 && isNotRepoMessage(error)) {
            return done(Buffer.from("false"));
          }
          fail(error);
        };
        parser = (text) => {
          return text.trim() === "true";
        };
      }
    });
    init_CleanSummary = __esm2({
      "src/lib/responses/CleanSummary.ts"() {
        "use strict";
        init_utils();
        CleanResponse = class {
          constructor(dryRun) {
            this.dryRun = dryRun;
            this.paths = [];
            this.files = [];
            this.folders = [];
          }
        };
        removalRegexp = /^[a-z]+\s*/i;
        dryRunRemovalRegexp = /^[a-z]+\s+[a-z]+\s*/i;
        isFolderRegexp = /\/$/;
      }
    });
    task_exports = {};
    __export2(task_exports, {
      EMPTY_COMMANDS: () => EMPTY_COMMANDS,
      adhocExecTask: () => adhocExecTask,
      configurationErrorTask: () => configurationErrorTask,
      isBufferTask: () => isBufferTask,
      isEmptyTask: () => isEmptyTask,
      straightThroughBufferTask: () => straightThroughBufferTask,
      straightThroughStringTask: () => straightThroughStringTask
    });
    init_task = __esm2({
      "src/lib/tasks/task.ts"() {
        "use strict";
        init_task_configuration_error();
        EMPTY_COMMANDS = [];
      }
    });
    clean_exports = {};
    __export2(clean_exports, {
      CONFIG_ERROR_INTERACTIVE_MODE: () => CONFIG_ERROR_INTERACTIVE_MODE,
      CONFIG_ERROR_MODE_REQUIRED: () => CONFIG_ERROR_MODE_REQUIRED,
      CONFIG_ERROR_UNKNOWN_OPTION: () => CONFIG_ERROR_UNKNOWN_OPTION,
      CleanOptions: () => CleanOptions,
      cleanTask: () => cleanTask,
      cleanWithOptionsTask: () => cleanWithOptionsTask,
      isCleanOptionsArray: () => isCleanOptionsArray
    });
    init_clean = __esm2({
      "src/lib/tasks/clean.ts"() {
        "use strict";
        init_CleanSummary();
        init_utils();
        init_task();
        CONFIG_ERROR_INTERACTIVE_MODE = "Git clean interactive mode is not supported";
        CONFIG_ERROR_MODE_REQUIRED = 'Git clean mode parameter ("n" or "f") is required';
        CONFIG_ERROR_UNKNOWN_OPTION = "Git clean unknown option found in: ";
        CleanOptions = /* @__PURE__ */ ((CleanOptions2) => {
          CleanOptions2["DRY_RUN"] = "n";
          CleanOptions2["FORCE"] = "f";
          CleanOptions2["IGNORED_INCLUDED"] = "x";
          CleanOptions2["IGNORED_ONLY"] = "X";
          CleanOptions2["EXCLUDING"] = "e";
          CleanOptions2["QUIET"] = "q";
          CleanOptions2["RECURSIVE"] = "d";
          return CleanOptions2;
        })(CleanOptions || {});
        CleanOptionValues = /* @__PURE__ */ new Set([
          "i",
          ...asStringArray(Object.values(CleanOptions))
        ]);
      }
    });
    init_ConfigList = __esm2({
      "src/lib/responses/ConfigList.ts"() {
        "use strict";
        init_utils();
        ConfigList = class {
          constructor() {
            this.files = [];
            this.values = /* @__PURE__ */ Object.create(null);
          }
          get all() {
            if (!this._all) {
              this._all = this.files.reduce((all, file) => {
                return Object.assign(all, this.values[file]);
              }, {});
            }
            return this._all;
          }
          addFile(file) {
            if (!(file in this.values)) {
              const latest = last(this.files);
              this.values[file] = latest ? Object.create(this.values[latest]) : {};
              this.files.push(file);
            }
            return this.values[file];
          }
          addValue(file, key, value) {
            const values = this.addFile(file);
            if (!Object.hasOwn(values, key)) {
              values[key] = value;
            } else if (Array.isArray(values[key])) {
              values[key].push(value);
            } else {
              values[key] = [values[key], value];
            }
            this._all = void 0;
          }
        };
      }
    });
    init_config = __esm2({
      "src/lib/tasks/config.ts"() {
        "use strict";
        init_ConfigList();
        init_utils();
        GitConfigScope = /* @__PURE__ */ ((GitConfigScope2) => {
          GitConfigScope2["system"] = "system";
          GitConfigScope2["global"] = "global";
          GitConfigScope2["local"] = "local";
          GitConfigScope2["worktree"] = "worktree";
          return GitConfigScope2;
        })(GitConfigScope || {});
      }
    });
    init_diff_name_status = __esm2({
      "src/lib/tasks/diff-name-status.ts"() {
        "use strict";
        DiffNameStatus = /* @__PURE__ */ ((DiffNameStatus2) => {
          DiffNameStatus2["ADDED"] = "A";
          DiffNameStatus2["COPIED"] = "C";
          DiffNameStatus2["DELETED"] = "D";
          DiffNameStatus2["MODIFIED"] = "M";
          DiffNameStatus2["RENAMED"] = "R";
          DiffNameStatus2["CHANGED"] = "T";
          DiffNameStatus2["UNMERGED"] = "U";
          DiffNameStatus2["UNKNOWN"] = "X";
          DiffNameStatus2["BROKEN"] = "B";
          return DiffNameStatus2;
        })(DiffNameStatus || {});
        diffNameStatus = new Set(Object.values(DiffNameStatus));
      }
    });
    init_grep = __esm2({
      "src/lib/tasks/grep.ts"() {
        "use strict";
        init_utils();
        init_task();
        disallowedOptions = ["-h"];
        Query = /* @__PURE__ */ Symbol("grepQuery");
        GrepQuery = class {
          constructor() {
            this[_a] = [];
          }
          *[(_a = Query, Symbol.iterator)]() {
            for (const query of this[Query]) {
              yield query;
            }
          }
          and(...and) {
            and.length && this[Query].push("--and", "(", ...prefixedArray(and, "-e"), ")");
            return this;
          }
          param(...param) {
            this[Query].push(...prefixedArray(param, "-e"));
            return this;
          }
        };
      }
    });
    reset_exports = {};
    __export2(reset_exports, {
      ResetMode: () => ResetMode,
      getResetMode: () => getResetMode,
      resetTask: () => resetTask
    });
    init_reset = __esm2({
      "src/lib/tasks/reset.ts"() {
        "use strict";
        init_utils();
        init_task();
        ResetMode = /* @__PURE__ */ ((ResetMode2) => {
          ResetMode2["MIXED"] = "mixed";
          ResetMode2["SOFT"] = "soft";
          ResetMode2["HARD"] = "hard";
          ResetMode2["MERGE"] = "merge";
          ResetMode2["KEEP"] = "keep";
          return ResetMode2;
        })(ResetMode || {});
        validResetModes = asStringArray(Object.values(ResetMode));
      }
    });
    init_git_logger = __esm2({
      "src/lib/git-logger.ts"() {
        "use strict";
        init_utils();
        import_debug.default.formatters.L = (value) => String(filterHasLength(value) ? value.length : "-");
        import_debug.default.formatters.B = (value) => {
          if (Buffer.isBuffer(value)) {
            return value.toString("utf8");
          }
          return objectToString(value);
        };
      }
    });
    init_tasks_pending_queue = __esm2({
      "src/lib/runners/tasks-pending-queue.ts"() {
        "use strict";
        init_git_error();
        init_git_logger();
        TasksPendingQueue = class _TasksPendingQueue {
          constructor(logLabel = "GitExecutor") {
            this.logLabel = logLabel;
            this._queue = /* @__PURE__ */ new Map();
          }
          withProgress(task) {
            return this._queue.get(task);
          }
          createProgress(task) {
            const name = _TasksPendingQueue.getName(task.commands[0]);
            const logger = createLogger(this.logLabel, name);
            return {
              task,
              logger,
              name
            };
          }
          push(task) {
            const progress = this.createProgress(task);
            progress.logger("Adding task to the queue, commands = %o", task.commands);
            this._queue.set(task, progress);
            return progress;
          }
          fatal(err) {
            for (const [task, { logger }] of Array.from(this._queue.entries())) {
              if (task === err.task) {
                logger.info(`Failed %o`, err);
                logger(
                  `Fatal exception, any as-yet un-started tasks run through this executor will not be attempted`
                );
              } else {
                logger.info(
                  `A fatal exception occurred in a previous task, the queue has been purged: %o`,
                  err.message
                );
              }
              this.complete(task);
            }
            if (this._queue.size !== 0) {
              throw new Error(`Queue size should be zero after fatal: ${this._queue.size}`);
            }
          }
          complete(task) {
            const progress = this.withProgress(task);
            if (progress) {
              this._queue.delete(task);
            }
          }
          attempt(task) {
            const progress = this.withProgress(task);
            if (!progress) {
              throw new GitError(void 0, "TasksPendingQueue: attempt called for an unknown task");
            }
            progress.logger("Starting task");
            return progress;
          }
          static getName(name = "empty") {
            return `task:${name}:${++_TasksPendingQueue.counter}`;
          }
          static {
            this.counter = 0;
          }
        };
      }
    });
    init_git_executor_chain = __esm2({
      "src/lib/runners/git-executor-chain.ts"() {
        "use strict";
        init_git_error();
        init_task();
        init_utils();
        init_tasks_pending_queue();
        GitExecutorChain = class {
          constructor(_executor, _scheduler, _plugins) {
            this._executor = _executor;
            this._scheduler = _scheduler;
            this._plugins = _plugins;
            this._chain = Promise.resolve();
            this._queue = new TasksPendingQueue();
          }
          get cwd() {
            return this._cwd || this._executor.cwd;
          }
          set cwd(cwd) {
            this._cwd = cwd;
          }
          get env() {
            return this._executor.env;
          }
          get outputHandler() {
            return this._executor.outputHandler;
          }
          chain() {
            return this;
          }
          push(task) {
            this._queue.push(task);
            return this._chain = this._chain.then(() => this.attemptTask(task));
          }
          async attemptTask(task) {
            const onScheduleComplete = await this._scheduler.next();
            const onQueueComplete = () => this._queue.complete(task);
            try {
              const { logger } = this._queue.attempt(task);
              return await (isEmptyTask(task) ? this.attemptEmptyTask(task, logger) : this.attemptRemoteTask(task, logger));
            } catch (e) {
              throw this.onFatalException(task, e);
            } finally {
              onQueueComplete();
              onScheduleComplete();
            }
          }
          onFatalException(task, e) {
            const gitError = e instanceof GitError ? Object.assign(e, { task }) : new GitError(task, e && String(e));
            this._chain = Promise.resolve();
            this._queue.fatal(gitError);
            return gitError;
          }
          async attemptRemoteTask(task, logger) {
            const binary = this._plugins.exec("spawn.binary", "", pluginContext(task, task.commands));
            const args = this._plugins.exec("spawn.args", [...task.commands], {
              ...pluginContext(task, task.commands),
              env: { ...this.env }
            });
            const raw = await this.gitResponse(
              task,
              binary,
              args,
              this.outputHandler,
              logger.step("SPAWN")
            );
            const outputStreams = await this.handleTaskData(task, args, raw, logger.step("HANDLE"));
            logger(`passing response to task's parser as a %s`, task.format);
            if (isBufferTask(task)) {
              return callTaskParser(task.parser, outputStreams);
            }
            return callTaskParser(task.parser, outputStreams.asStrings());
          }
          async attemptEmptyTask(task, logger) {
            logger(`empty task bypassing child process to call to task's parser`);
            return task.parser(this);
          }
          handleTaskData(task, args, result, logger) {
            const { exitCode, rejection, stdOut, stdErr } = result;
            return new Promise((done, fail) => {
              logger(`Preparing to handle process response exitCode=%d stdOut=`, exitCode);
              const { error } = this._plugins.exec(
                "task.error",
                { error: rejection },
                {
                  ...pluginContext(task, args),
                  ...result
                }
              );
              if (error && task.onError) {
                logger.info(`exitCode=%s handling with custom error handler`);
                return task.onError(
                  result,
                  error,
                  (newStdOut) => {
                    logger.info(`custom error handler treated as success`);
                    logger(`custom error returned a %s`, objectToString(newStdOut));
                    done(
                      new GitOutputStreams(
                        Array.isArray(newStdOut) ? Buffer.concat(newStdOut) : newStdOut,
                        Buffer.concat(stdErr)
                      )
                    );
                  },
                  fail
                );
              }
              if (error) {
                logger.info(
                  `handling as error: exitCode=%s stdErr=%s rejection=%o`,
                  exitCode,
                  stdErr.length,
                  rejection
                );
                return fail(error);
              }
              logger.info(`retrieving task output complete`);
              done(new GitOutputStreams(Buffer.concat(stdOut), Buffer.concat(stdErr)));
            });
          }
          async gitResponse(task, command, args, outputHandler, logger) {
            const outputLogger = logger.sibling("output");
            const spawnOptions = this._plugins.exec(
              "spawn.options",
              {
                cwd: this.cwd,
                env: this.env,
                windowsHide: true
              },
              pluginContext(task, task.commands)
            );
            return new Promise((done) => {
              const stdOut = [];
              const stdErr = [];
              logger.info(`%s %o`, command, args);
              logger("%O", spawnOptions);
              let rejection = this._beforeSpawn(task, args);
              if (rejection) {
                return done({
                  stdOut,
                  stdErr,
                  exitCode: 9901,
                  rejection
                });
              }
              this._plugins.exec("spawn.before", void 0, {
                ...pluginContext(task, args),
                kill(reason) {
                  rejection = reason || rejection;
                }
              });
              const spawned = spawn(command, args, spawnOptions);
              spawned.stdout.on(
                "data",
                onDataReceived(stdOut, "stdOut", logger, outputLogger.step("stdOut"))
              );
              spawned.stderr.on(
                "data",
                onDataReceived(stdErr, "stdErr", logger, outputLogger.step("stdErr"))
              );
              spawned.on("error", onErrorReceived(stdErr, logger));
              if (outputHandler) {
                logger(`Passing child process stdOut/stdErr to custom outputHandler`);
                outputHandler(command, spawned.stdout, spawned.stderr, [...args]);
              }
              this._plugins.exec("spawn.after", void 0, {
                ...pluginContext(task, args),
                spawned,
                close(exitCode, reason) {
                  done({
                    stdOut,
                    stdErr,
                    exitCode,
                    rejection: rejection || reason
                  });
                },
                kill(reason) {
                  if (spawned.killed) {
                    return;
                  }
                  rejection = reason;
                  spawned.kill("SIGINT");
                }
              });
            });
          }
          _beforeSpawn(task, args) {
            let rejection;
            this._plugins.exec("spawn.before", void 0, {
              ...pluginContext(task, args),
              kill(reason) {
                rejection = reason || rejection;
              }
            });
            return rejection;
          }
        };
      }
    });
    git_executor_exports = {};
    __export2(git_executor_exports, {
      GitExecutor: () => GitExecutor
    });
    init_git_executor = __esm2({
      "src/lib/runners/git-executor.ts"() {
        "use strict";
        init_git_executor_chain();
        GitExecutor = class {
          constructor(cwd, _scheduler, _plugins) {
            this.cwd = cwd;
            this._scheduler = _scheduler;
            this._plugins = _plugins;
            this._chain = new GitExecutorChain(this, this._scheduler, this._plugins);
          }
          chain() {
            return new GitExecutorChain(this, this._scheduler, this._plugins);
          }
          push(task) {
            return this._chain.push(task);
          }
        };
      }
    });
    init_task_callback = __esm2({
      "src/lib/task-callback.ts"() {
        "use strict";
        init_git_response_error();
        init_utils();
      }
    });
    init_change_working_directory = __esm2({
      "src/lib/tasks/change-working-directory.ts"() {
        "use strict";
        init_utils();
        init_task();
      }
    });
    init_checkout = __esm2({
      "src/lib/tasks/checkout.ts"() {
        "use strict";
        init_utils();
        init_task();
      }
    });
    init_count_objects = __esm2({
      "src/lib/tasks/count-objects.ts"() {
        "use strict";
        init_utils();
        parser2 = new LineParser(
          /([a-z-]+): (\d+)$/,
          (result, [key, value]) => {
            const property = asCamelCase(key);
            if (Object.hasOwn(result, property)) {
              result[property] = asNumber(value);
            }
          }
        );
      }
    });
    init_parse_commit = __esm2({
      "src/lib/parsers/parse-commit.ts"() {
        "use strict";
        init_utils();
        parsers = [
          new LineParser(/^\[([^\s]+)( \([^)]+\))? ([^\]]+)/, (result, [branch, root, commit]) => {
            result.branch = branch;
            result.commit = commit;
            result.root = !!root;
          }),
          new LineParser(/\s*Author:\s(.+)/i, (result, [author]) => {
            const parts = author.split("<");
            const email = parts.pop();
            if (!email || !email.includes("@")) {
              return;
            }
            result.author = {
              email: email.substr(0, email.length - 1),
              name: parts.join("<").trim()
            };
          }),
          new LineParser(
            /(\d+)[^,]*(?:,\s*(\d+)[^,]*)(?:,\s*(\d+))/g,
            (result, [changes, insertions, deletions]) => {
              result.summary.changes = parseInt(changes, 10) || 0;
              result.summary.insertions = parseInt(insertions, 10) || 0;
              result.summary.deletions = parseInt(deletions, 10) || 0;
            }
          ),
          new LineParser(
            /^(\d+)[^,]*(?:,\s*(\d+)[^(]+\(([+-]))?/,
            (result, [changes, lines, direction]) => {
              result.summary.changes = parseInt(changes, 10) || 0;
              const count = parseInt(lines, 10) || 0;
              if (direction === "-") {
                result.summary.deletions = count;
              } else if (direction === "+") {
                result.summary.insertions = count;
              }
            }
          )
        ];
      }
    });
    init_commit = __esm2({
      "src/lib/tasks/commit.ts"() {
        "use strict";
        init_parse_commit();
        init_utils();
        init_task();
      }
    });
    init_first_commit = __esm2({
      "src/lib/tasks/first-commit.ts"() {
        "use strict";
        init_utils();
        init_task();
      }
    });
    init_hash_object = __esm2({
      "src/lib/tasks/hash-object.ts"() {
        "use strict";
        init_task();
      }
    });
    init_InitSummary = __esm2({
      "src/lib/responses/InitSummary.ts"() {
        "use strict";
        InitSummary = class {
          constructor(bare, path2, existing, gitDir) {
            this.bare = bare;
            this.path = path2;
            this.existing = existing;
            this.gitDir = gitDir;
          }
        };
        initResponseRegex = /^Init.+ repository in (.+)$/;
        reInitResponseRegex = /^Rein.+ in (.+)$/;
      }
    });
    init_init = __esm2({
      "src/lib/tasks/init.ts"() {
        "use strict";
        init_InitSummary();
        bareCommand = "--bare";
      }
    });
    init_log_format = __esm2({
      "src/lib/args/log-format.ts"() {
        "use strict";
        logFormatRegex = /^--(stat|numstat|name-only|name-status)(=|$)/;
      }
    });
    init_DiffSummary = __esm2({
      "src/lib/responses/DiffSummary.ts"() {
        "use strict";
        DiffSummary = class {
          constructor() {
            this.changed = 0;
            this.deletions = 0;
            this.insertions = 0;
            this.files = [];
          }
        };
      }
    });
    init_parse_diff_summary = __esm2({
      "src/lib/parsers/parse-diff-summary.ts"() {
        "use strict";
        init_log_format();
        init_DiffSummary();
        init_diff_name_status();
        init_utils();
        statParser = [
          new LineParser(
            /^(.+)\s+\|\s+(\d+)(\s+[+\-]+)?$/,
            (result, [file, changes, alterations = ""]) => {
              result.files.push({
                file: file.trim(),
                changes: asNumber(changes),
                insertions: alterations.replace(/[^+]/g, "").length,
                deletions: alterations.replace(/[^-]/g, "").length,
                binary: false
              });
            }
          ),
          new LineParser(
            /^(.+) \|\s+Bin ([0-9.]+) -> ([0-9.]+) ([a-z]+)/,
            (result, [file, before, after]) => {
              result.files.push({
                file: file.trim(),
                before: asNumber(before),
                after: asNumber(after),
                binary: true
              });
            }
          ),
          new LineParser(
            /(\d+) files? changed\s*((?:, \d+ [^,]+){0,2})/,
            (result, [changed, summary]) => {
              const inserted = /(\d+) i/.exec(summary);
              const deleted = /(\d+) d/.exec(summary);
              result.changed = asNumber(changed);
              result.insertions = asNumber(inserted?.[1]);
              result.deletions = asNumber(deleted?.[1]);
            }
          )
        ];
        numStatParser = [
          new LineParser(
            /(\d+)\t(\d+)\t(.+)$/,
            (result, [changesInsert, changesDelete, file]) => {
              const insertions = asNumber(changesInsert);
              const deletions = asNumber(changesDelete);
              result.changed++;
              result.insertions += insertions;
              result.deletions += deletions;
              result.files.push({
                file,
                changes: insertions + deletions,
                insertions,
                deletions,
                binary: false
              });
            }
          ),
          new LineParser(/-\t-\t(.+)$/, (result, [file]) => {
            result.changed++;
            result.files.push({
              file,
              after: 0,
              before: 0,
              binary: true
            });
          })
        ];
        nameOnlyParser = [
          new LineParser(/(.+)$/, (result, [file]) => {
            result.changed++;
            result.files.push({
              file,
              changes: 0,
              insertions: 0,
              deletions: 0,
              binary: false
            });
          })
        ];
        nameStatusParser = [
          new LineParser(
            /([ACDMRTUXB])([0-9]{0,3})\t(.[^\t]*)(\t(.[^\t]*))?$/,
            (result, [status, similarity, from, _to, to]) => {
              result.changed++;
              result.files.push({
                file: to ?? from,
                changes: 0,
                insertions: 0,
                deletions: 0,
                binary: false,
                status: orVoid(isDiffNameStatus(status) && status),
                from: orVoid(!!to && from !== to && from),
                similarity: asNumber(similarity)
              });
            }
          )
        ];
        diffSummaryParsers = {
          [
            ""
            /* NONE */
          ]: statParser,
          [
            "--stat"
            /* STAT */
          ]: statParser,
          [
            "--numstat"
            /* NUM_STAT */
          ]: numStatParser,
          [
            "--name-status"
            /* NAME_STATUS */
          ]: nameStatusParser,
          [
            "--name-only"
            /* NAME_ONLY */
          ]: nameOnlyParser
        };
      }
    });
    init_parse_list_log_summary = __esm2({
      "src/lib/parsers/parse-list-log-summary.ts"() {
        "use strict";
        init_utils();
        init_parse_diff_summary();
        init_log_format();
        START_BOUNDARY = "\xF2\xF2\xF2\xF2\xF2\xF2 ";
        COMMIT_BOUNDARY = " \xF2\xF2";
        SPLITTER = " \xF2 ";
        defaultFieldNames = ["hash", "date", "message", "refs", "author_name", "author_email"];
      }
    });
    diff_exports = {};
    __export2(diff_exports, {
      diffSummaryTask: () => diffSummaryTask,
      validateLogFormatConfig: () => validateLogFormatConfig
    });
    init_diff = __esm2({
      "src/lib/tasks/diff.ts"() {
        "use strict";
        init_log_format();
        init_parse_diff_summary();
        init_task();
      }
    });
    init_log = __esm2({
      "src/lib/tasks/log.ts"() {
        "use strict";
        init_log_format();
        init_parse_list_log_summary();
        init_utils();
        init_task();
        init_diff();
        excludeOptions = /* @__PURE__ */ ((excludeOptions2) => {
          excludeOptions2[excludeOptions2["--pretty"] = 0] = "--pretty";
          excludeOptions2[excludeOptions2["max-count"] = 1] = "max-count";
          excludeOptions2[excludeOptions2["maxCount"] = 2] = "maxCount";
          excludeOptions2[excludeOptions2["n"] = 3] = "n";
          excludeOptions2[excludeOptions2["file"] = 4] = "file";
          excludeOptions2[excludeOptions2["format"] = 5] = "format";
          excludeOptions2[excludeOptions2["from"] = 6] = "from";
          excludeOptions2[excludeOptions2["to"] = 7] = "to";
          excludeOptions2[excludeOptions2["splitter"] = 8] = "splitter";
          excludeOptions2[excludeOptions2["symmetric"] = 9] = "symmetric";
          excludeOptions2[excludeOptions2["mailMap"] = 10] = "mailMap";
          excludeOptions2[excludeOptions2["multiLine"] = 11] = "multiLine";
          excludeOptions2[excludeOptions2["strictDate"] = 12] = "strictDate";
          return excludeOptions2;
        })(excludeOptions || {});
      }
    });
    init_MergeSummary = __esm2({
      "src/lib/responses/MergeSummary.ts"() {
        "use strict";
        MergeSummaryConflict = class {
          constructor(reason, file = null, meta) {
            this.reason = reason;
            this.file = file;
            this.meta = meta;
          }
          toString() {
            return `${this.file}:${this.reason}`;
          }
        };
        MergeSummaryDetail = class {
          constructor() {
            this.conflicts = [];
            this.merges = [];
            this.result = "success";
          }
          get failed() {
            return this.conflicts.length > 0;
          }
          get reason() {
            return this.result;
          }
          toString() {
            if (this.conflicts.length) {
              return `CONFLICTS: ${this.conflicts.join(", ")}`;
            }
            return "OK";
          }
        };
      }
    });
    init_PullSummary = __esm2({
      "src/lib/responses/PullSummary.ts"() {
        "use strict";
        PullSummary = class {
          constructor() {
            this.remoteMessages = {
              all: []
            };
            this.created = [];
            this.deleted = [];
            this.files = [];
            this.deletions = {};
            this.insertions = {};
            this.summary = {
              changes: 0,
              deletions: 0,
              insertions: 0
            };
          }
        };
        PullFailedSummary = class {
          constructor() {
            this.remote = "";
            this.hash = {
              local: "",
              remote: ""
            };
            this.branch = {
              local: "",
              remote: ""
            };
            this.message = "";
          }
          toString() {
            return this.message;
          }
        };
      }
    });
    init_parse_remote_objects = __esm2({
      "src/lib/parsers/parse-remote-objects.ts"() {
        "use strict";
        init_utils();
        remoteMessagesObjectParsers = [
          new RemoteLineParser(
            /^remote:\s*(enumerating|counting|compressing) objects: (\d+),/i,
            (result, [action, count]) => {
              const key = action.toLowerCase();
              const enumeration = objectEnumerationResult(result.remoteMessages);
              Object.assign(enumeration, { [key]: asNumber(count) });
            }
          ),
          new RemoteLineParser(
            /^remote:\s*(enumerating|counting|compressing) objects: \d+% \(\d+\/(\d+)\),/i,
            (result, [action, count]) => {
              const key = action.toLowerCase();
              const enumeration = objectEnumerationResult(result.remoteMessages);
              Object.assign(enumeration, { [key]: asNumber(count) });
            }
          ),
          new RemoteLineParser(
            /total ([^,]+), reused ([^,]+), pack-reused (\d+)/i,
            (result, [total, reused, packReused]) => {
              const objects = objectEnumerationResult(result.remoteMessages);
              objects.total = asObjectCount(total);
              objects.reused = asObjectCount(reused);
              objects.packReused = asNumber(packReused);
            }
          )
        ];
      }
    });
    init_parse_remote_messages = __esm2({
      "src/lib/parsers/parse-remote-messages.ts"() {
        "use strict";
        init_utils();
        init_parse_remote_objects();
        parsers2 = [
          new RemoteLineParser(/^remote:\s*(.+)$/, (result, [text]) => {
            result.remoteMessages.all.push(text.trim());
            return false;
          }),
          ...remoteMessagesObjectParsers,
          new RemoteLineParser(
            [/create a (?:pull|merge) request/i, /\s(https?:\/\/\S+)$/],
            (result, [pullRequestUrl]) => {
              result.remoteMessages.pullRequestUrl = pullRequestUrl;
            }
          ),
          new RemoteLineParser(
            [/found (\d+) vulnerabilities.+\(([^)]+)\)/i, /\s(https?:\/\/\S+)$/],
            (result, [count, summary, url]) => {
              result.remoteMessages.vulnerabilities = {
                count: asNumber(count),
                summary,
                url
              };
            }
          )
        ];
        RemoteMessageSummary = class {
          constructor() {
            this.all = [];
          }
        };
      }
    });
    init_parse_pull = __esm2({
      "src/lib/parsers/parse-pull.ts"() {
        "use strict";
        init_PullSummary();
        init_utils();
        init_parse_remote_messages();
        FILE_UPDATE_REGEX = /^\s*(.+?)\s+\|\s+\d+\s*(\+*)(-*)/;
        SUMMARY_REGEX = /(\d+)\D+((\d+)\D+\(\+\))?(\D+(\d+)\D+\(-\))?/;
        ACTION_REGEX = /^(create|delete) mode \d+ (.+)/;
        parsers3 = [
          new LineParser(FILE_UPDATE_REGEX, (result, [file, insertions, deletions]) => {
            result.files.push(file);
            if (insertions) {
              result.insertions[file] = insertions.length;
            }
            if (deletions) {
              result.deletions[file] = deletions.length;
            }
          }),
          new LineParser(SUMMARY_REGEX, (result, [changes, , insertions, , deletions]) => {
            if (insertions !== void 0 || deletions !== void 0) {
              result.summary.changes = +changes || 0;
              result.summary.insertions = +insertions || 0;
              result.summary.deletions = +deletions || 0;
              return true;
            }
            return false;
          }),
          new LineParser(ACTION_REGEX, (result, [action, file]) => {
            append(result.files, file);
            append(action === "create" ? result.created : result.deleted, file);
          })
        ];
        errorParsers = [
          new LineParser(/^from\s(.+)$/i, (result, [remote]) => void (result.remote = remote)),
          new LineParser(/^fatal:\s(.+)$/, (result, [message]) => void (result.message = message)),
          new LineParser(
            /([a-z0-9]+)\.\.([a-z0-9]+)\s+(\S+)\s+->\s+(\S+)$/,
            (result, [hashLocal, hashRemote, branchLocal, branchRemote]) => {
              result.branch.local = branchLocal;
              result.hash.local = hashLocal;
              result.branch.remote = branchRemote;
              result.hash.remote = hashRemote;
            }
          )
        ];
        parsePullDetail = (stdOut, stdErr) => {
          return parseStringResponse(new PullSummary(), parsers3, [stdOut, stdErr]);
        };
        parsePullResult = (stdOut, stdErr) => {
          return Object.assign(
            new PullSummary(),
            parsePullDetail(stdOut, stdErr),
            parseRemoteMessages(stdOut, stdErr)
          );
        };
      }
    });
    init_parse_merge = __esm2({
      "src/lib/parsers/parse-merge.ts"() {
        "use strict";
        init_MergeSummary();
        init_utils();
        init_parse_pull();
        parsers4 = [
          new LineParser(/^Auto-merging\s+(.+)$/, (summary, [autoMerge]) => {
            summary.merges.push(autoMerge);
          }),
          new LineParser(/^CONFLICT\s+\((.+)\): Merge conflict in (.+)$/, (summary, [reason, file]) => {
            summary.conflicts.push(new MergeSummaryConflict(reason, file));
          }),
          new LineParser(
            /^CONFLICT\s+\((.+\/delete)\): (.+) deleted in (.+) and/,
            (summary, [reason, file, deleteRef]) => {
              summary.conflicts.push(new MergeSummaryConflict(reason, file, { deleteRef }));
            }
          ),
          new LineParser(/^CONFLICT\s+\((.+)\):/, (summary, [reason]) => {
            summary.conflicts.push(new MergeSummaryConflict(reason, null));
          }),
          new LineParser(/^Automatic merge failed;\s+(.+)$/, (summary, [result]) => {
            summary.result = result;
          })
        ];
        parseMergeResult = (stdOut, stdErr) => {
          return Object.assign(parseMergeDetail(stdOut, stdErr), parsePullResult(stdOut, stdErr));
        };
        parseMergeDetail = (stdOut) => {
          return parseStringResponse(new MergeSummaryDetail(), parsers4, stdOut);
        };
      }
    });
    init_merge = __esm2({
      "src/lib/tasks/merge.ts"() {
        "use strict";
        init_git_response_error();
        init_parse_merge();
        init_task();
      }
    });
    init_parse_push = __esm2({
      "src/lib/parsers/parse-push.ts"() {
        "use strict";
        init_utils();
        init_parse_remote_messages();
        parsers5 = [
          new LineParser(/^Pushing to (.+)$/, (result, [repo]) => {
            result.repo = repo;
          }),
          new LineParser(/^updating local tracking ref '(.+)'/, (result, [local]) => {
            result.ref = {
              ...result.ref || {},
              local
            };
          }),
          new LineParser(/^[=*-]\s+([^:]+):(\S+)\s+\[(.+)]$/, (result, [local, remote, type]) => {
            result.pushed.push(pushResultPushedItem(local, remote, type));
          }),
          new LineParser(
            /^Branch '([^']+)' set up to track remote branch '([^']+)' from '([^']+)'/,
            (result, [local, remote, remoteName]) => {
              result.branch = {
                ...result.branch || {},
                local,
                remote,
                remoteName
              };
            }
          ),
          new LineParser(
            /^([^:]+):(\S+)\s+([a-z0-9]+)\.\.([a-z0-9]+)$/,
            (result, [local, remote, from, to]) => {
              result.update = {
                head: {
                  local,
                  remote
                },
                hash: {
                  from,
                  to
                }
              };
            }
          )
        ];
        parsePushResult = (stdOut, stdErr) => {
          const pushDetail = parsePushDetail(stdOut, stdErr);
          const responseDetail = parseRemoteMessages(stdOut, stdErr);
          return {
            ...pushDetail,
            ...responseDetail
          };
        };
        parsePushDetail = (stdOut, stdErr) => {
          return parseStringResponse({ pushed: [] }, parsers5, [stdOut, stdErr]);
        };
      }
    });
    push_exports = {};
    __export2(push_exports, {
      pushTagsTask: () => pushTagsTask,
      pushTask: () => pushTask
    });
    init_push = __esm2({
      "src/lib/tasks/push.ts"() {
        "use strict";
        init_parse_push();
        init_utils();
      }
    });
    init_show = __esm2({
      "src/lib/tasks/show.ts"() {
        "use strict";
        init_utils();
        init_task();
      }
    });
    init_FileStatusSummary = __esm2({
      "src/lib/responses/FileStatusSummary.ts"() {
        "use strict";
        fromPathRegex = /^(.+)\0(.+)$/;
        FileStatusSummary = class {
          constructor(path2, index, working_dir) {
            this.path = path2;
            this.index = index;
            this.working_dir = working_dir;
            if (index === "R" || working_dir === "R") {
              const detail = fromPathRegex.exec(path2) || [null, path2, path2];
              this.from = detail[2] || "";
              this.path = detail[1] || "";
            }
          }
        };
      }
    });
    init_StatusSummary = __esm2({
      "src/lib/responses/StatusSummary.ts"() {
        "use strict";
        init_utils();
        init_FileStatusSummary();
        StatusSummary = class {
          constructor() {
            this.not_added = [];
            this.conflicted = [];
            this.created = [];
            this.deleted = [];
            this.ignored = void 0;
            this.modified = [];
            this.renamed = [];
            this.files = [];
            this.staged = [];
            this.ahead = 0;
            this.behind = 0;
            this.current = null;
            this.tracking = null;
            this.detached = false;
            this.isClean = () => {
              return !this.files.length;
            };
          }
        };
        parsers6 = new Map([
          parser3(
            " ",
            "A",
            (result, file) => result.created.push(file)
          ),
          parser3(
            " ",
            "D",
            (result, file) => result.deleted.push(file)
          ),
          parser3(
            " ",
            "M",
            (result, file) => result.modified.push(file)
          ),
          parser3("A", " ", (result, file) => {
            result.created.push(file);
            result.staged.push(file);
          }),
          parser3("A", "M", (result, file) => {
            result.created.push(file);
            result.staged.push(file);
            result.modified.push(file);
          }),
          parser3("D", " ", (result, file) => {
            result.deleted.push(file);
            result.staged.push(file);
          }),
          parser3("M", " ", (result, file) => {
            result.modified.push(file);
            result.staged.push(file);
          }),
          parser3("M", "M", (result, file) => {
            result.modified.push(file);
            result.staged.push(file);
          }),
          parser3("R", " ", (result, file) => {
            result.renamed.push(renamedFile(file));
          }),
          parser3("R", "M", (result, file) => {
            const renamed = renamedFile(file);
            result.renamed.push(renamed);
            result.modified.push(renamed.to);
          }),
          parser3("!", "!", (_result, _file) => {
            (_result.ignored = _result.ignored || []).push(_file);
          }),
          parser3(
            "?",
            "?",
            (result, file) => result.not_added.push(file)
          ),
          ...conflicts(
            "A",
            "A",
            "U"
            /* UNMERGED */
          ),
          ...conflicts(
            "D",
            "D",
            "U"
            /* UNMERGED */
          ),
          ...conflicts(
            "U",
            "A",
            "D",
            "U"
            /* UNMERGED */
          ),
          [
            "##",
            (result, line) => {
              const aheadReg = /ahead (\d+)/;
              const behindReg = /behind (\d+)/;
              const currentReg = /^(.+?(?=(?:\.{3}|\s|$)))/;
              const trackingReg = /\.{3}(\S*)/;
              const onEmptyBranchReg = /\son\s(\S+?)(?=\.{3}|$)/;
              let regexResult = aheadReg.exec(line);
              result.ahead = regexResult && +regexResult[1] || 0;
              regexResult = behindReg.exec(line);
              result.behind = regexResult && +regexResult[1] || 0;
              regexResult = currentReg.exec(line);
              result.current = filterType(regexResult?.[1], filterString, null);
              regexResult = trackingReg.exec(line);
              result.tracking = filterType(regexResult?.[1], filterString, null);
              regexResult = onEmptyBranchReg.exec(line);
              if (regexResult) {
                result.current = filterType(regexResult?.[1], filterString, result.current);
              }
              result.detached = /\(no branch\)/.test(line);
            }
          ]
        ]);
        parseStatusSummary = function(text) {
          const lines = text.split(NULL);
          const status = new StatusSummary();
          for (let i2 = 0, l = lines.length; i2 < l; ) {
            let line = lines[i2++].trim();
            if (!line) {
              continue;
            }
            if (line.charAt(0) === "R") {
              line += NULL + (lines[i2++] || "");
            }
            splitLine(status, line);
          }
          return status;
        };
      }
    });
    init_status = __esm2({
      "src/lib/tasks/status.ts"() {
        "use strict";
        init_StatusSummary();
        ignoredOptions = ["--null", "-z"];
      }
    });
    init_version = __esm2({
      "src/lib/tasks/version.ts"() {
        "use strict";
        init_utils();
        NOT_INSTALLED = "installed=false";
        parsers7 = [
          new LineParser(
            /version (\d+)\.(\d+)\.(\d+)(?:\s*\((.+)\))?/,
            (result, [major, minor, patch, agent = ""]) => {
              Object.assign(
                result,
                versionResponse(asNumber(major), asNumber(minor), asNumber(patch), agent)
              );
            }
          ),
          new LineParser(
            /version (\d+)\.(\d+)\.(\D+)(.+)?$/,
            (result, [major, minor, patch, agent = ""]) => {
              Object.assign(result, versionResponse(asNumber(major), asNumber(minor), patch, agent));
            }
          )
        ];
      }
    });
    init_clone = __esm2({
      "src/lib/tasks/clone.ts"() {
        "use strict";
        init_task();
        init_utils();
        cloneTask = (repo, directory, customArgs) => {
          const commands = ["clone", ...customArgs];
          filterString(repo) && commands.push(c(repo));
          filterString(directory) && commands.push(c(directory));
          return straightThroughStringTask(commands);
        };
        cloneMirrorTask = (repo, directory, customArgs) => {
          append(customArgs, "--mirror");
          return cloneTask(repo, directory, customArgs);
        };
      }
    });
    simple_git_api_exports = {};
    __export2(simple_git_api_exports, {
      SimpleGitApi: () => SimpleGitApi
    });
    init_simple_git_api = __esm2({
      "src/lib/simple-git-api.ts"() {
        "use strict";
        init_task_callback();
        init_change_working_directory();
        init_checkout();
        init_count_objects();
        init_commit();
        init_config();
        init_first_commit();
        init_grep();
        init_hash_object();
        init_init();
        init_log();
        init_merge();
        init_push();
        init_show();
        init_status();
        init_task();
        init_version();
        init_utils();
        init_clone();
        SimpleGitApi = class {
          constructor(_executor) {
            this._executor = _executor;
          }
          _runTask(task, then) {
            const chain = this._executor.chain();
            const promise = chain.push(task);
            if (then) {
              taskCallback(task, promise, then);
            }
            return Object.create(this, {
              then: { value: promise.then.bind(promise) },
              catch: { value: promise.catch.bind(promise) },
              _executor: { value: chain }
            });
          }
          add(files) {
            return this._runTask(
              straightThroughStringTask(["add", ...asArray(files)]),
              trailingFunctionArgument(arguments)
            );
          }
          cwd(directory) {
            const next = trailingFunctionArgument(arguments);
            if (typeof directory === "string") {
              return this._runTask(changeWorkingDirectoryTask(directory, this._executor), next);
            }
            if (typeof directory?.path === "string") {
              return this._runTask(
                changeWorkingDirectoryTask(
                  directory.path,
                  directory.root && this._executor || void 0
                ),
                next
              );
            }
            return this._runTask(
              configurationErrorTask("Git.cwd: workingDirectory must be supplied as a string"),
              next
            );
          }
          hashObject(path2, write) {
            return this._runTask(
              hashObjectTask(path2, write === true),
              trailingFunctionArgument(arguments)
            );
          }
          init(bare) {
            return this._runTask(
              initTask(bare === true, this._executor.cwd, getTrailingOptions(arguments)),
              trailingFunctionArgument(arguments)
            );
          }
          merge() {
            return this._runTask(
              mergeTask(getTrailingOptions(arguments)),
              trailingFunctionArgument(arguments)
            );
          }
          mergeFromTo(remote, branch) {
            if (!(filterString(remote) && filterString(branch))) {
              return this._runTask(
                configurationErrorTask(
                  `Git.mergeFromTo requires that the 'remote' and 'branch' arguments are supplied as strings`
                )
              );
            }
            return this._runTask(
              mergeTask([remote, branch, ...getTrailingOptions(arguments)]),
              trailingFunctionArgument(arguments, false)
            );
          }
          outputHandler(handler) {
            this._executor.outputHandler = handler;
            return this;
          }
          push() {
            const task = pushTask(
              {
                remote: filterType(arguments[0], filterString),
                branch: filterType(arguments[1], filterString)
              },
              getTrailingOptions(arguments)
            );
            return this._runTask(task, trailingFunctionArgument(arguments));
          }
          stash() {
            return this._runTask(
              straightThroughStringTask(["stash", ...getTrailingOptions(arguments)]),
              trailingFunctionArgument(arguments)
            );
          }
          status() {
            return this._runTask(
              statusTask(getTrailingOptions(arguments)),
              trailingFunctionArgument(arguments)
            );
          }
        };
        Object.assign(
          SimpleGitApi.prototype,
          checkout_default(),
          clone_default(),
          commit_default(),
          config_default(),
          count_objects_default(),
          first_commit_default(),
          grep_default(),
          log_default(),
          show_default(),
          version_default()
        );
      }
    });
    scheduler_exports = {};
    __export2(scheduler_exports, {
      Scheduler: () => Scheduler
    });
    init_scheduler = __esm2({
      "src/lib/runners/scheduler.ts"() {
        "use strict";
        init_utils();
        init_git_logger();
        createScheduledTask = /* @__PURE__ */ (() => {
          let id = 0;
          return () => {
            id++;
            const { promise, done } = (0, import_promise_deferred.createDeferred)();
            return {
              promise,
              done,
              id
            };
          };
        })();
        Scheduler = class {
          constructor(concurrency = 2) {
            this.concurrency = concurrency;
            this.logger = createLogger("", "scheduler");
            this.pending = [];
            this.running = [];
            this.logger(`Constructed, concurrency=%s`, concurrency);
          }
          schedule() {
            if (!this.pending.length || this.running.length >= this.concurrency) {
              this.logger(
                `Schedule attempt ignored, pending=%s running=%s concurrency=%s`,
                this.pending.length,
                this.running.length,
                this.concurrency
              );
              return;
            }
            const task = append(this.running, this.pending.shift());
            this.logger(`Attempting id=%s`, task.id);
            task.done(() => {
              this.logger(`Completing id=`, task.id);
              remove(this.running, task);
              this.schedule();
            });
          }
          next() {
            const { promise, id } = append(this.pending, createScheduledTask());
            this.logger(`Scheduling id=%s`, id);
            this.schedule();
            return promise;
          }
        };
      }
    });
    apply_patch_exports = {};
    __export2(apply_patch_exports, {
      applyPatchTask: () => applyPatchTask
    });
    init_apply_patch = __esm2({
      "src/lib/tasks/apply-patch.ts"() {
        "use strict";
        init_task();
      }
    });
    init_BranchDeleteSummary = __esm2({
      "src/lib/responses/BranchDeleteSummary.ts"() {
        "use strict";
        BranchDeletionBatch = class {
          constructor() {
            this.all = [];
            this.branches = {};
            this.errors = [];
          }
          get success() {
            return !this.errors.length;
          }
        };
      }
    });
    init_parse_branch_delete = __esm2({
      "src/lib/parsers/parse-branch-delete.ts"() {
        "use strict";
        init_BranchDeleteSummary();
        init_utils();
        deleteSuccessRegex = /(\S+)\s+\(\S+\s([^)]+)\)/;
        deleteErrorRegex = /^error[^']+'([^']+)'/m;
        parsers8 = [
          new LineParser(deleteSuccessRegex, (result, [branch, hash]) => {
            const deletion = branchDeletionSuccess(branch, hash);
            result.all.push(deletion);
            result.branches[branch] = deletion;
          }),
          new LineParser(deleteErrorRegex, (result, [branch]) => {
            const deletion = branchDeletionFailure(branch);
            result.errors.push(deletion);
            result.all.push(deletion);
            result.branches[branch] = deletion;
          })
        ];
        parseBranchDeletions = (stdOut, stdErr) => {
          return parseStringResponse(new BranchDeletionBatch(), parsers8, [stdOut, stdErr]);
        };
      }
    });
    init_BranchSummary = __esm2({
      "src/lib/responses/BranchSummary.ts"() {
        "use strict";
        BranchSummaryResult = class {
          constructor() {
            this.all = [];
            this.branches = {};
            this.current = "";
            this.detached = false;
          }
          push(status, detached, name, commit, label) {
            if (status === "*") {
              this.detached = detached;
              this.current = name;
            }
            this.all.push(name);
            this.branches[name] = {
              current: status === "*",
              linkedWorkTree: status === "+",
              name,
              commit,
              label
            };
          }
        };
      }
    });
    init_parse_branch = __esm2({
      "src/lib/parsers/parse-branch.ts"() {
        "use strict";
        init_BranchSummary();
        init_utils();
        parsers9 = [
          new LineParser(
            /^([*+]\s)?\((?:HEAD )?detached (?:from|at) (\S+)\)\s+([a-z0-9]+)\s(.*)$/,
            (result, [current, name, commit, label]) => {
              result.push(branchStatus(current), true, name, commit, label);
            }
          ),
          new LineParser(
            /^([*+]\s)?(\S+)\s+([a-z0-9]+)\s?(.*)$/s,
            (result, [current, name, commit, label]) => {
              result.push(branchStatus(current), false, name, commit, label);
            }
          )
        ];
        currentBranchParser = new LineParser(/^(\S+)$/s, (result, [name]) => {
          result.push("*", false, name, "", "");
        });
      }
    });
    branch_exports = {};
    __export2(branch_exports, {
      branchLocalTask: () => branchLocalTask,
      branchTask: () => branchTask,
      containsDeleteBranchCommand: () => containsDeleteBranchCommand,
      deleteBranchTask: () => deleteBranchTask,
      deleteBranchesTask: () => deleteBranchesTask
    });
    init_branch = __esm2({
      "src/lib/tasks/branch.ts"() {
        "use strict";
        init_git_response_error();
        init_parse_branch_delete();
        init_parse_branch();
        init_utils();
      }
    });
    init_CheckIgnore = __esm2({
      "src/lib/responses/CheckIgnore.ts"() {
        "use strict";
        parseCheckIgnore = (text) => {
          return text.split(/\n/g).map(toPath).filter(Boolean);
        };
      }
    });
    check_ignore_exports = {};
    __export2(check_ignore_exports, {
      checkIgnoreTask: () => checkIgnoreTask
    });
    init_check_ignore = __esm2({
      "src/lib/tasks/check-ignore.ts"() {
        "use strict";
        init_CheckIgnore();
      }
    });
    init_parse_fetch = __esm2({
      "src/lib/parsers/parse-fetch.ts"() {
        "use strict";
        init_utils();
        parsers10 = [
          new LineParser(/From (.+)$/, (result, [remote]) => {
            result.remote = remote;
          }),
          new LineParser(/\* \[new branch]\s+(\S+)\s*-> (.+)$/, (result, [name, tracking]) => {
            result.branches.push({
              name,
              tracking
            });
          }),
          new LineParser(/\* \[new tag]\s+(\S+)\s*-> (.+)$/, (result, [name, tracking]) => {
            result.tags.push({
              name,
              tracking
            });
          }),
          new LineParser(/- \[deleted]\s+\S+\s*-> (.+)$/, (result, [tracking]) => {
            result.deleted.push({
              tracking
            });
          }),
          new LineParser(
            /\s*([^.]+)\.\.(\S+)\s+(\S+)\s*-> (.+)$/,
            (result, [from, to, name, tracking]) => {
              result.updated.push({
                name,
                tracking,
                to,
                from
              });
            }
          )
        ];
      }
    });
    fetch_exports = {};
    __export2(fetch_exports, {
      fetchTask: () => fetchTask
    });
    init_fetch = __esm2({
      "src/lib/tasks/fetch.ts"() {
        "use strict";
        init_parse_fetch();
        init_task();
      }
    });
    init_parse_move = __esm2({
      "src/lib/parsers/parse-move.ts"() {
        "use strict";
        init_utils();
        parsers11 = [
          new LineParser(/^Renaming (.+) to (.+)$/, (result, [from, to]) => {
            result.moves.push({ from, to });
          })
        ];
      }
    });
    move_exports = {};
    __export2(move_exports, {
      moveTask: () => moveTask
    });
    init_move = __esm2({
      "src/lib/tasks/move.ts"() {
        "use strict";
        init_parse_move();
        init_utils();
      }
    });
    pull_exports = {};
    __export2(pull_exports, {
      pullTask: () => pullTask
    });
    init_pull = __esm2({
      "src/lib/tasks/pull.ts"() {
        "use strict";
        init_git_response_error();
        init_parse_pull();
        init_utils();
      }
    });
    init_GetRemoteSummary = __esm2({
      "src/lib/responses/GetRemoteSummary.ts"() {
        "use strict";
        init_utils();
      }
    });
    remote_exports = {};
    __export2(remote_exports, {
      addRemoteTask: () => addRemoteTask,
      getRemotesTask: () => getRemotesTask,
      listRemotesTask: () => listRemotesTask,
      remoteTask: () => remoteTask,
      removeRemoteTask: () => removeRemoteTask
    });
    init_remote = __esm2({
      "src/lib/tasks/remote.ts"() {
        "use strict";
        init_GetRemoteSummary();
        init_task();
      }
    });
    stash_list_exports = {};
    __export2(stash_list_exports, {
      stashListTask: () => stashListTask
    });
    init_stash_list = __esm2({
      "src/lib/tasks/stash-list.ts"() {
        "use strict";
        init_log_format();
        init_parse_list_log_summary();
        init_diff();
        init_log();
      }
    });
    sub_module_exports = {};
    __export2(sub_module_exports, {
      addSubModuleTask: () => addSubModuleTask,
      initSubModuleTask: () => initSubModuleTask,
      subModuleTask: () => subModuleTask,
      updateSubModuleTask: () => updateSubModuleTask
    });
    init_sub_module = __esm2({
      "src/lib/tasks/sub-module.ts"() {
        "use strict";
        init_task();
      }
    });
    init_TagList = __esm2({
      "src/lib/responses/TagList.ts"() {
        "use strict";
        TagList = class {
          constructor(all, latest) {
            this.all = all;
            this.latest = latest;
          }
        };
        parseTagList = function(data, customSort = false) {
          const tags = data.split("\n").map(trimmed).filter(Boolean);
          if (!customSort) {
            tags.sort(function(tagA, tagB) {
              const partsA = tagA.split(".");
              const partsB = tagB.split(".");
              if (partsA.length === 1 || partsB.length === 1) {
                return singleSorted(toNumber(partsA[0]), toNumber(partsB[0]));
              }
              for (let i2 = 0, l = Math.max(partsA.length, partsB.length); i2 < l; i2++) {
                const diff = sorted(toNumber(partsA[i2]), toNumber(partsB[i2]));
                if (diff) {
                  return diff;
                }
              }
              return 0;
            });
          }
          const latest = customSort ? tags[0] : [...tags].reverse().find((tag) => tag.indexOf(".") >= 0);
          return new TagList(tags, latest);
        };
      }
    });
    tag_exports = {};
    __export2(tag_exports, {
      addAnnotatedTagTask: () => addAnnotatedTagTask,
      addTagTask: () => addTagTask,
      tagListTask: () => tagListTask
    });
    init_tag = __esm2({
      "src/lib/tasks/tag.ts"() {
        "use strict";
        init_TagList();
      }
    });
    require_git = __commonJS2({
      "src/git.js"(exports, module) {
        "use strict";
        var { GitExecutor: GitExecutor2 } = (init_git_executor(), __toCommonJS(git_executor_exports));
        var { SimpleGitApi: SimpleGitApi2 } = (init_simple_git_api(), __toCommonJS(simple_git_api_exports));
        var { Scheduler: Scheduler2 } = (init_scheduler(), __toCommonJS(scheduler_exports));
        var { adhocExecTask: adhocExecTask2, configurationErrorTask: configurationErrorTask2 } = (init_task(), __toCommonJS(task_exports));
        var {
          asArray: asArray2,
          filterArray: filterArray2,
          filterPrimitives: filterPrimitives2,
          filterString: filterString2,
          filterStringOrStringArray: filterStringOrStringArray2,
          filterType: filterType2,
          getTrailingOptions: getTrailingOptions2,
          trailingFunctionArgument: trailingFunctionArgument2,
          trailingOptionsArgument: trailingOptionsArgument2
        } = (init_utils(), __toCommonJS(utils_exports));
        var { applyPatchTask: applyPatchTask2 } = (init_apply_patch(), __toCommonJS(apply_patch_exports));
        var {
          branchTask: branchTask2,
          branchLocalTask: branchLocalTask2,
          deleteBranchesTask: deleteBranchesTask2,
          deleteBranchTask: deleteBranchTask2
        } = (init_branch(), __toCommonJS(branch_exports));
        var { checkIgnoreTask: checkIgnoreTask2 } = (init_check_ignore(), __toCommonJS(check_ignore_exports));
        var { checkIsRepoTask: checkIsRepoTask2 } = (init_check_is_repo(), __toCommonJS(check_is_repo_exports));
        var { cleanWithOptionsTask: cleanWithOptionsTask2, isCleanOptionsArray: isCleanOptionsArray2 } = (init_clean(), __toCommonJS(clean_exports));
        var { diffSummaryTask: diffSummaryTask2 } = (init_diff(), __toCommonJS(diff_exports));
        var { fetchTask: fetchTask2 } = (init_fetch(), __toCommonJS(fetch_exports));
        var { moveTask: moveTask2 } = (init_move(), __toCommonJS(move_exports));
        var { pullTask: pullTask2 } = (init_pull(), __toCommonJS(pull_exports));
        var { pushTagsTask: pushTagsTask2 } = (init_push(), __toCommonJS(push_exports));
        var {
          addRemoteTask: addRemoteTask2,
          getRemotesTask: getRemotesTask2,
          listRemotesTask: listRemotesTask2,
          remoteTask: remoteTask2,
          removeRemoteTask: removeRemoteTask2
        } = (init_remote(), __toCommonJS(remote_exports));
        var { getResetMode: getResetMode2, resetTask: resetTask2 } = (init_reset(), __toCommonJS(reset_exports));
        var { stashListTask: stashListTask2 } = (init_stash_list(), __toCommonJS(stash_list_exports));
        var {
          addSubModuleTask: addSubModuleTask2,
          initSubModuleTask: initSubModuleTask2,
          subModuleTask: subModuleTask2,
          updateSubModuleTask: updateSubModuleTask2
        } = (init_sub_module(), __toCommonJS(sub_module_exports));
        var { addAnnotatedTagTask: addAnnotatedTagTask2, addTagTask: addTagTask2, tagListTask: tagListTask2 } = (init_tag(), __toCommonJS(tag_exports));
        var { straightThroughBufferTask: straightThroughBufferTask2, straightThroughStringTask: straightThroughStringTask2 } = (init_task(), __toCommonJS(task_exports));
        function Git2(options, plugins) {
          this._plugins = plugins;
          this._executor = new GitExecutor2(
            options.baseDir,
            new Scheduler2(options.maxConcurrentProcesses),
            plugins
          );
          this._trimmed = options.trimmed;
        }
        (Git2.prototype = Object.create(SimpleGitApi2.prototype)).constructor = Git2;
        Git2.prototype.customBinary = function(command) {
          this._plugins.reconfigure("binary", command);
          return this;
        };
        Git2.prototype.env = function(name, value) {
          if (arguments.length === 1 && typeof name === "object") {
            this._executor.env = name;
          } else {
            (this._executor.env = this._executor.env || {})[name] = value;
          }
          return this;
        };
        Git2.prototype.stashList = function(options) {
          return this._runTask(
            stashListTask2(
              trailingOptionsArgument2(arguments) || {},
              filterArray2(options) && options || []
            ),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.mv = function(from, to) {
          return this._runTask(moveTask2(from, to), trailingFunctionArgument2(arguments));
        };
        Git2.prototype.checkoutLatestTag = function(then) {
          var git = this;
          return this.pull(function() {
            git.tags(function(err, tags) {
              git.checkout(tags.latest, then);
            });
          });
        };
        Git2.prototype.pull = function(remote, branch, options, then) {
          return this._runTask(
            pullTask2(
              filterType2(remote, filterString2),
              filterType2(branch, filterString2),
              getTrailingOptions2(arguments)
            ),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.fetch = function(remote, branch) {
          return this._runTask(
            fetchTask2(
              filterType2(remote, filterString2),
              filterType2(branch, filterString2),
              getTrailingOptions2(arguments)
            ),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.silent = function(silence) {
          return this._runTask(
            adhocExecTask2(
              () => console.warn(
                "simple-git deprecation notice: git.silent: logging should be configured using the `debug` library / `DEBUG` environment variable, this method will be removed."
              )
            )
          );
        };
        Git2.prototype.tags = function(options, then) {
          return this._runTask(
            tagListTask2(getTrailingOptions2(arguments)),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.rebase = function() {
          return this._runTask(
            straightThroughStringTask2(["rebase", ...getTrailingOptions2(arguments)]),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.reset = function(mode) {
          return this._runTask(
            resetTask2(getResetMode2(mode), getTrailingOptions2(arguments)),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.revert = function(commit) {
          const next = trailingFunctionArgument2(arguments);
          if (typeof commit !== "string") {
            return this._runTask(configurationErrorTask2("Commit must be a string"), next);
          }
          return this._runTask(
            straightThroughStringTask2(["revert", ...getTrailingOptions2(arguments, 0, true), commit]),
            next
          );
        };
        Git2.prototype.addTag = function(name) {
          const task = typeof name === "string" ? addTagTask2(name) : configurationErrorTask2("Git.addTag requires a tag name");
          return this._runTask(task, trailingFunctionArgument2(arguments));
        };
        Git2.prototype.addAnnotatedTag = function(tagName, tagMessage) {
          return this._runTask(
            addAnnotatedTagTask2(tagName, tagMessage),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.deleteLocalBranch = function(branchName, forceDelete, then) {
          return this._runTask(
            deleteBranchTask2(branchName, typeof forceDelete === "boolean" ? forceDelete : false),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.deleteLocalBranches = function(branchNames, forceDelete, then) {
          return this._runTask(
            deleteBranchesTask2(branchNames, typeof forceDelete === "boolean" ? forceDelete : false),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.branch = function(options, then) {
          return this._runTask(
            branchTask2(getTrailingOptions2(arguments)),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.branchLocal = function(then) {
          return this._runTask(branchLocalTask2(), trailingFunctionArgument2(arguments));
        };
        Git2.prototype.raw = function(commands) {
          const createRestCommands = !Array.isArray(commands);
          const command = [].slice.call(createRestCommands ? arguments : commands, 0);
          for (let i2 = 0; i2 < command.length && createRestCommands; i2++) {
            if (!filterPrimitives2(command[i2])) {
              command.splice(i2, command.length - i2);
              break;
            }
          }
          command.push(...getTrailingOptions2(arguments, 0, true));
          var next = trailingFunctionArgument2(arguments);
          if (!command.length) {
            return this._runTask(
              configurationErrorTask2("Raw: must supply one or more command to execute"),
              next
            );
          }
          return this._runTask(straightThroughStringTask2(command, this._trimmed), next);
        };
        Git2.prototype.submoduleAdd = function(repo, path2, then) {
          return this._runTask(addSubModuleTask2(repo, path2), trailingFunctionArgument2(arguments));
        };
        Git2.prototype.submoduleUpdate = function(args, then) {
          return this._runTask(
            updateSubModuleTask2(getTrailingOptions2(arguments, true)),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.submoduleInit = function(args, then) {
          return this._runTask(
            initSubModuleTask2(getTrailingOptions2(arguments, true)),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.subModule = function(options, then) {
          return this._runTask(
            subModuleTask2(getTrailingOptions2(arguments)),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.listRemote = function() {
          return this._runTask(
            listRemotesTask2(getTrailingOptions2(arguments)),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.addRemote = function(remoteName, remoteRepo, then) {
          return this._runTask(
            addRemoteTask2(remoteName, remoteRepo, getTrailingOptions2(arguments)),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.removeRemote = function(remoteName, then) {
          return this._runTask(removeRemoteTask2(remoteName), trailingFunctionArgument2(arguments));
        };
        Git2.prototype.getRemotes = function(verbose, then) {
          return this._runTask(getRemotesTask2(verbose === true), trailingFunctionArgument2(arguments));
        };
        Git2.prototype.remote = function(options, then) {
          return this._runTask(
            remoteTask2(getTrailingOptions2(arguments)),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.tag = function(options, then) {
          const command = getTrailingOptions2(arguments);
          if (command[0] !== "tag") {
            command.unshift("tag");
          }
          return this._runTask(straightThroughStringTask2(command), trailingFunctionArgument2(arguments));
        };
        Git2.prototype.updateServerInfo = function(then) {
          return this._runTask(
            straightThroughStringTask2(["update-server-info"]),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.pushTags = function(remote, then) {
          const task = pushTagsTask2(
            { remote: filterType2(remote, filterString2) },
            getTrailingOptions2(arguments)
          );
          return this._runTask(task, trailingFunctionArgument2(arguments));
        };
        Git2.prototype.rm = function(files) {
          return this._runTask(
            straightThroughStringTask2(["rm", "-f", ...asArray2(files)]),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.rmKeepLocal = function(files) {
          return this._runTask(
            straightThroughStringTask2(["rm", "--cached", ...asArray2(files)]),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.catFile = function(options, then) {
          return this._catFile("utf-8", arguments);
        };
        Git2.prototype.binaryCatFile = function() {
          return this._catFile("buffer", arguments);
        };
        Git2.prototype._catFile = function(format, args) {
          var handler = trailingFunctionArgument2(args);
          var command = ["cat-file"];
          var options = args[0];
          if (typeof options === "string") {
            return this._runTask(
              configurationErrorTask2("Git.catFile: options must be supplied as an array of strings"),
              handler
            );
          }
          if (Array.isArray(options)) {
            command.push.apply(command, options);
          }
          const task = format === "buffer" ? straightThroughBufferTask2(command) : straightThroughStringTask2(command);
          return this._runTask(task, handler);
        };
        Git2.prototype.diff = function(options, then) {
          const task = filterString2(options) ? configurationErrorTask2(
            "git.diff: supplying options as a single string is no longer supported, switch to an array of strings"
          ) : straightThroughStringTask2(["diff", ...getTrailingOptions2(arguments)]);
          return this._runTask(task, trailingFunctionArgument2(arguments));
        };
        Git2.prototype.diffSummary = function() {
          return this._runTask(
            diffSummaryTask2(getTrailingOptions2(arguments, 1)),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.applyPatch = function(patches) {
          const task = !filterStringOrStringArray2(patches) ? configurationErrorTask2(
            `git.applyPatch requires one or more string patches as the first argument`
          ) : applyPatchTask2(asArray2(patches), getTrailingOptions2([].slice.call(arguments, 1)));
          return this._runTask(task, trailingFunctionArgument2(arguments));
        };
        Git2.prototype.revparse = function() {
          const commands = ["rev-parse", ...getTrailingOptions2(arguments, true)];
          return this._runTask(
            straightThroughStringTask2(commands, true),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.clean = function(mode, options, then) {
          const usingCleanOptionsArray = isCleanOptionsArray2(mode);
          const cleanMode = usingCleanOptionsArray && mode.join("") || filterType2(mode, filterString2) || "";
          const customArgs = getTrailingOptions2([].slice.call(arguments, usingCleanOptionsArray ? 1 : 0));
          return this._runTask(
            cleanWithOptionsTask2(cleanMode, customArgs),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.exec = function(then) {
          const task = {
            commands: [],
            format: "utf-8",
            parser() {
              if (typeof then === "function") {
                then();
              }
            }
          };
          return this._runTask(task);
        };
        Git2.prototype.clearQueue = function() {
          return this._runTask(
            adhocExecTask2(
              () => console.warn(
                "simple-git deprecation notice: clearQueue() is deprecated and will be removed, switch to using the abortPlugin instead."
              )
            )
          );
        };
        Git2.prototype.checkIgnore = function(pathnames, then) {
          return this._runTask(
            checkIgnoreTask2(asArray2(filterType2(pathnames, filterStringOrStringArray2, []))),
            trailingFunctionArgument2(arguments)
          );
        };
        Git2.prototype.checkIsRepo = function(checkType, then) {
          return this._runTask(
            checkIsRepoTask2(filterType2(checkType, filterString2)),
            trailingFunctionArgument2(arguments)
          );
        };
        module.exports = Git2;
      }
    });
    init_git_error();
    GitConstructError = class extends GitError {
      constructor(config, message) {
        super(void 0, message);
        this.config = config;
      }
    };
    init_git_error();
    init_git_error();
    GitPluginError = class extends GitError {
      constructor(task, plugin, message) {
        super(task, message);
        this.task = task;
        this.plugin = plugin;
        Object.setPrototypeOf(this, new.target.prototype);
      }
    };
    init_git_response_error();
    init_task_configuration_error();
    init_check_is_repo();
    init_clean();
    init_config();
    init_diff_name_status();
    init_grep();
    init_reset();
    init_utils();
    init_utils();
    never = (0, import_promise_deferred2.deferred)().promise;
    init_utils();
    WRONG_NUMBER_ERR = `Invalid value supplied for custom binary, requires a single string or an array containing either one or two strings`;
    WRONG_CHARS_ERR = `Invalid value supplied for custom binary, restricted characters must be removed or supply the unsafe.allowUnsafeCustomBinary option`;
    init_git_error();
    init_utils();
    PluginStore = class {
      constructor() {
        this.plugins = /* @__PURE__ */ new Set();
        this.events = new EventEmitter();
      }
      on(type, listener) {
        this.events.on(type, listener);
      }
      reconfigure(type, data) {
        this.events.emit(type, data);
      }
      append(type, action) {
        const plugin = append(this.plugins, { type, action });
        return () => this.plugins.delete(plugin);
      }
      add(plugin) {
        const plugins = [];
        asArray(plugin).forEach((plugin2) => plugin2 && this.plugins.add(append(plugins, plugin2)));
        return () => {
          plugins.forEach((plugin2) => this.plugins.delete(plugin2));
        };
      }
      exec(type, data, context) {
        let output = data;
        const contextual = Object.freeze(Object.create(context));
        for (const plugin of this.plugins) {
          if (plugin.type === type) {
            output = plugin.action(output, contextual);
          }
        }
        return output;
      }
    };
    init_utils();
    init_utils();
    init_utils();
    Git = require_git();
    init_git_response_error();
    functionNamesBuilderApi = ["customBinary", "env", "outputHandler", "silent"];
    functionNamesPromiseApi = [
      "add",
      "addAnnotatedTag",
      "addConfig",
      "addRemote",
      "addTag",
      "applyPatch",
      "binaryCatFile",
      "branch",
      "branchLocal",
      "catFile",
      "checkIgnore",
      "checkIsRepo",
      "checkout",
      "checkoutBranch",
      "checkoutLatestTag",
      "checkoutLocalBranch",
      "clean",
      "clone",
      "commit",
      "cwd",
      "deleteLocalBranch",
      "deleteLocalBranches",
      "diff",
      "diffSummary",
      "exec",
      "fetch",
      "getRemotes",
      "init",
      "listConfig",
      "listRemote",
      "log",
      "merge",
      "mergeFromTo",
      "mirror",
      "mv",
      "pull",
      "push",
      "pushTags",
      "raw",
      "rebase",
      "remote",
      "removeRemote",
      "reset",
      "revert",
      "revparse",
      "rm",
      "rmKeepLocal",
      "show",
      "stash",
      "stashList",
      "status",
      "subModule",
      "submoduleAdd",
      "submoduleInit",
      "submoduleUpdate",
      "tag",
      "tags",
      "updateServerInfo"
    ];
    simpleGit = gitInstanceFactory;
    esm_default = gitInstanceFactory;
  }
});

// node_modules/commander/esm.mjs
var import_index = __toESM(require_commander(), 1);
var {
  program,
  createCommand,
  createArgument,
  createOption,
  CommanderError,
  InvalidArgumentError,
  InvalidOptionArgumentError,
  // deprecated old name
  Command,
  Argument,
  Option,
  Help
} = import_index.default;

// node_modules/chalk/source/vendor/ansi-styles/index.js
var ANSI_BACKGROUND_OFFSET = 10;
var wrapAnsi16 = (offset = 0) => (code) => `\x1B[${code + offset}m`;
var wrapAnsi256 = (offset = 0) => (code) => `\x1B[${38 + offset};5;${code}m`;
var wrapAnsi16m = (offset = 0) => (red, green, blue) => `\x1B[${38 + offset};2;${red};${green};${blue}m`;
var styles = {
  modifier: {
    reset: [0, 0],
    // 21 isn't widely supported and 22 does the same thing
    bold: [1, 22],
    dim: [2, 22],
    italic: [3, 23],
    underline: [4, 24],
    overline: [53, 55],
    inverse: [7, 27],
    hidden: [8, 28],
    strikethrough: [9, 29]
  },
  color: {
    black: [30, 39],
    red: [31, 39],
    green: [32, 39],
    yellow: [33, 39],
    blue: [34, 39],
    magenta: [35, 39],
    cyan: [36, 39],
    white: [37, 39],
    // Bright color
    blackBright: [90, 39],
    gray: [90, 39],
    // Alias of `blackBright`
    grey: [90, 39],
    // Alias of `blackBright`
    redBright: [91, 39],
    greenBright: [92, 39],
    yellowBright: [93, 39],
    blueBright: [94, 39],
    magentaBright: [95, 39],
    cyanBright: [96, 39],
    whiteBright: [97, 39]
  },
  bgColor: {
    bgBlack: [40, 49],
    bgRed: [41, 49],
    bgGreen: [42, 49],
    bgYellow: [43, 49],
    bgBlue: [44, 49],
    bgMagenta: [45, 49],
    bgCyan: [46, 49],
    bgWhite: [47, 49],
    // Bright color
    bgBlackBright: [100, 49],
    bgGray: [100, 49],
    // Alias of `bgBlackBright`
    bgGrey: [100, 49],
    // Alias of `bgBlackBright`
    bgRedBright: [101, 49],
    bgGreenBright: [102, 49],
    bgYellowBright: [103, 49],
    bgBlueBright: [104, 49],
    bgMagentaBright: [105, 49],
    bgCyanBright: [106, 49],
    bgWhiteBright: [107, 49]
  }
};
var modifierNames = Object.keys(styles.modifier);
var foregroundColorNames = Object.keys(styles.color);
var backgroundColorNames = Object.keys(styles.bgColor);
var colorNames = [...foregroundColorNames, ...backgroundColorNames];
function assembleStyles() {
  const codes = /* @__PURE__ */ new Map();
  for (const [groupName, group] of Object.entries(styles)) {
    for (const [styleName, style] of Object.entries(group)) {
      styles[styleName] = {
        open: `\x1B[${style[0]}m`,
        close: `\x1B[${style[1]}m`
      };
      group[styleName] = styles[styleName];
      codes.set(style[0], style[1]);
    }
    Object.defineProperty(styles, groupName, {
      value: group,
      enumerable: false
    });
  }
  Object.defineProperty(styles, "codes", {
    value: codes,
    enumerable: false
  });
  styles.color.close = "\x1B[39m";
  styles.bgColor.close = "\x1B[49m";
  styles.color.ansi = wrapAnsi16();
  styles.color.ansi256 = wrapAnsi256();
  styles.color.ansi16m = wrapAnsi16m();
  styles.bgColor.ansi = wrapAnsi16(ANSI_BACKGROUND_OFFSET);
  styles.bgColor.ansi256 = wrapAnsi256(ANSI_BACKGROUND_OFFSET);
  styles.bgColor.ansi16m = wrapAnsi16m(ANSI_BACKGROUND_OFFSET);
  Object.defineProperties(styles, {
    rgbToAnsi256: {
      value(red, green, blue) {
        if (red === green && green === blue) {
          if (red < 8) {
            return 16;
          }
          if (red > 248) {
            return 231;
          }
          return Math.round((red - 8) / 247 * 24) + 232;
        }
        return 16 + 36 * Math.round(red / 255 * 5) + 6 * Math.round(green / 255 * 5) + Math.round(blue / 255 * 5);
      },
      enumerable: false
    },
    hexToRgb: {
      value(hex) {
        const matches = /[a-f\d]{6}|[a-f\d]{3}/i.exec(hex.toString(16));
        if (!matches) {
          return [0, 0, 0];
        }
        let [colorString] = matches;
        if (colorString.length === 3) {
          colorString = [...colorString].map((character) => character + character).join("");
        }
        const integer = Number.parseInt(colorString, 16);
        return [
          /* eslint-disable no-bitwise */
          integer >> 16 & 255,
          integer >> 8 & 255,
          integer & 255
          /* eslint-enable no-bitwise */
        ];
      },
      enumerable: false
    },
    hexToAnsi256: {
      value: (hex) => styles.rgbToAnsi256(...styles.hexToRgb(hex)),
      enumerable: false
    },
    ansi256ToAnsi: {
      value(code) {
        if (code < 8) {
          return 30 + code;
        }
        if (code < 16) {
          return 90 + (code - 8);
        }
        let red;
        let green;
        let blue;
        if (code >= 232) {
          red = ((code - 232) * 10 + 8) / 255;
          green = red;
          blue = red;
        } else {
          code -= 16;
          const remainder = code % 36;
          red = Math.floor(code / 36) / 5;
          green = Math.floor(remainder / 6) / 5;
          blue = remainder % 6 / 5;
        }
        const value = Math.max(red, green, blue) * 2;
        if (value === 0) {
          return 30;
        }
        let result = 30 + (Math.round(blue) << 2 | Math.round(green) << 1 | Math.round(red));
        if (value === 2) {
          result += 60;
        }
        return result;
      },
      enumerable: false
    },
    rgbToAnsi: {
      value: (red, green, blue) => styles.ansi256ToAnsi(styles.rgbToAnsi256(red, green, blue)),
      enumerable: false
    },
    hexToAnsi: {
      value: (hex) => styles.ansi256ToAnsi(styles.hexToAnsi256(hex)),
      enumerable: false
    }
  });
  return styles;
}
var ansiStyles = assembleStyles();
var ansi_styles_default = ansiStyles;

// node_modules/chalk/source/vendor/supports-color/index.js
import process2 from "node:process";
import os from "node:os";
import tty from "node:tty";
function hasFlag(flag, argv = globalThis.Deno ? globalThis.Deno.args : process2.argv) {
  const prefix = flag.startsWith("-") ? "" : flag.length === 1 ? "-" : "--";
  const position = argv.indexOf(prefix + flag);
  const terminatorPosition = argv.indexOf("--");
  return position !== -1 && (terminatorPosition === -1 || position < terminatorPosition);
}
var { env } = process2;
var flagForceColor;
if (hasFlag("no-color") || hasFlag("no-colors") || hasFlag("color=false") || hasFlag("color=never")) {
  flagForceColor = 0;
} else if (hasFlag("color") || hasFlag("colors") || hasFlag("color=true") || hasFlag("color=always")) {
  flagForceColor = 1;
}
function envForceColor() {
  if ("FORCE_COLOR" in env) {
    if (env.FORCE_COLOR === "true") {
      return 1;
    }
    if (env.FORCE_COLOR === "false") {
      return 0;
    }
    return env.FORCE_COLOR.length === 0 ? 1 : Math.min(Number.parseInt(env.FORCE_COLOR, 10), 3);
  }
}
function translateLevel(level) {
  if (level === 0) {
    return false;
  }
  return {
    level,
    hasBasic: true,
    has256: level >= 2,
    has16m: level >= 3
  };
}
function _supportsColor(haveStream, { streamIsTTY, sniffFlags = true } = {}) {
  const noFlagForceColor = envForceColor();
  if (noFlagForceColor !== void 0) {
    flagForceColor = noFlagForceColor;
  }
  const forceColor = sniffFlags ? flagForceColor : noFlagForceColor;
  if (forceColor === 0) {
    return 0;
  }
  if (sniffFlags) {
    if (hasFlag("color=16m") || hasFlag("color=full") || hasFlag("color=truecolor")) {
      return 3;
    }
    if (hasFlag("color=256")) {
      return 2;
    }
  }
  if ("TF_BUILD" in env && "AGENT_NAME" in env) {
    return 1;
  }
  if (haveStream && !streamIsTTY && forceColor === void 0) {
    return 0;
  }
  const min = forceColor || 0;
  if (env.TERM === "dumb") {
    return min;
  }
  if (process2.platform === "win32") {
    const osRelease = os.release().split(".");
    if (Number(osRelease[0]) >= 10 && Number(osRelease[2]) >= 10586) {
      return Number(osRelease[2]) >= 14931 ? 3 : 2;
    }
    return 1;
  }
  if ("CI" in env) {
    if (["GITHUB_ACTIONS", "GITEA_ACTIONS", "CIRCLECI"].some((key) => key in env)) {
      return 3;
    }
    if (["TRAVIS", "APPVEYOR", "GITLAB_CI", "BUILDKITE", "DRONE"].some((sign) => sign in env) || env.CI_NAME === "codeship") {
      return 1;
    }
    return min;
  }
  if ("TEAMCITY_VERSION" in env) {
    return /^(9\.(0*[1-9]\d*)\.|\d{2,}\.)/.test(env.TEAMCITY_VERSION) ? 1 : 0;
  }
  if (env.COLORTERM === "truecolor") {
    return 3;
  }
  if (env.TERM === "xterm-kitty") {
    return 3;
  }
  if (env.TERM === "xterm-ghostty") {
    return 3;
  }
  if (env.TERM === "wezterm") {
    return 3;
  }
  if ("TERM_PROGRAM" in env) {
    const version = Number.parseInt((env.TERM_PROGRAM_VERSION || "").split(".")[0], 10);
    switch (env.TERM_PROGRAM) {
      case "iTerm.app": {
        return version >= 3 ? 3 : 2;
      }
      case "Apple_Terminal": {
        return 2;
      }
    }
  }
  if (/-256(color)?$/i.test(env.TERM)) {
    return 2;
  }
  if (/^screen|^xterm|^vt100|^vt220|^rxvt|color|ansi|cygwin|linux/i.test(env.TERM)) {
    return 1;
  }
  if ("COLORTERM" in env) {
    return 1;
  }
  return min;
}
function createSupportsColor(stream, options = {}) {
  const level = _supportsColor(stream, {
    streamIsTTY: stream && stream.isTTY,
    ...options
  });
  return translateLevel(level);
}
var supportsColor = {
  stdout: createSupportsColor({ isTTY: tty.isatty(1) }),
  stderr: createSupportsColor({ isTTY: tty.isatty(2) })
};
var supports_color_default = supportsColor;

// node_modules/chalk/source/utilities.js
function stringReplaceAll(string, substring, replacer) {
  let index = string.indexOf(substring);
  if (index === -1) {
    return string;
  }
  const substringLength = substring.length;
  let endIndex = 0;
  let returnValue = "";
  do {
    returnValue += string.slice(endIndex, index) + substring + replacer;
    endIndex = index + substringLength;
    index = string.indexOf(substring, endIndex);
  } while (index !== -1);
  returnValue += string.slice(endIndex);
  return returnValue;
}
function stringEncaseCRLFWithFirstIndex(string, prefix, postfix, index) {
  let endIndex = 0;
  let returnValue = "";
  do {
    const gotCR = string[index - 1] === "\r";
    returnValue += string.slice(endIndex, gotCR ? index - 1 : index) + prefix + (gotCR ? "\r\n" : "\n") + postfix;
    endIndex = index + 1;
    index = string.indexOf("\n", endIndex);
  } while (index !== -1);
  returnValue += string.slice(endIndex);
  return returnValue;
}

// node_modules/chalk/source/index.js
var { stdout: stdoutColor, stderr: stderrColor } = supports_color_default;
var GENERATOR = /* @__PURE__ */ Symbol("GENERATOR");
var STYLER = /* @__PURE__ */ Symbol("STYLER");
var IS_EMPTY = /* @__PURE__ */ Symbol("IS_EMPTY");
var levelMapping = [
  "ansi",
  "ansi",
  "ansi256",
  "ansi16m"
];
var styles2 = /* @__PURE__ */ Object.create(null);
var applyOptions = (object, options = {}) => {
  if (options.level && !(Number.isInteger(options.level) && options.level >= 0 && options.level <= 3)) {
    throw new Error("The `level` option should be an integer from 0 to 3");
  }
  const colorLevel = stdoutColor ? stdoutColor.level : 0;
  object.level = options.level === void 0 ? colorLevel : options.level;
};
var chalkFactory = (options) => {
  const chalk2 = (...strings) => strings.join(" ");
  applyOptions(chalk2, options);
  Object.setPrototypeOf(chalk2, createChalk.prototype);
  return chalk2;
};
function createChalk(options) {
  return chalkFactory(options);
}
Object.setPrototypeOf(createChalk.prototype, Function.prototype);
for (const [styleName, style] of Object.entries(ansi_styles_default)) {
  styles2[styleName] = {
    get() {
      const builder = createBuilder(this, createStyler(style.open, style.close, this[STYLER]), this[IS_EMPTY]);
      Object.defineProperty(this, styleName, { value: builder });
      return builder;
    }
  };
}
styles2.visible = {
  get() {
    const builder = createBuilder(this, this[STYLER], true);
    Object.defineProperty(this, "visible", { value: builder });
    return builder;
  }
};
var getModelAnsi = (model, level, type, ...arguments_) => {
  if (model === "rgb") {
    if (level === "ansi16m") {
      return ansi_styles_default[type].ansi16m(...arguments_);
    }
    if (level === "ansi256") {
      return ansi_styles_default[type].ansi256(ansi_styles_default.rgbToAnsi256(...arguments_));
    }
    return ansi_styles_default[type].ansi(ansi_styles_default.rgbToAnsi(...arguments_));
  }
  if (model === "hex") {
    return getModelAnsi("rgb", level, type, ...ansi_styles_default.hexToRgb(...arguments_));
  }
  return ansi_styles_default[type][model](...arguments_);
};
var usedModels = ["rgb", "hex", "ansi256"];
for (const model of usedModels) {
  styles2[model] = {
    get() {
      const { level } = this;
      return function(...arguments_) {
        const styler = createStyler(getModelAnsi(model, levelMapping[level], "color", ...arguments_), ansi_styles_default.color.close, this[STYLER]);
        return createBuilder(this, styler, this[IS_EMPTY]);
      };
    }
  };
  const bgModel = "bg" + model[0].toUpperCase() + model.slice(1);
  styles2[bgModel] = {
    get() {
      const { level } = this;
      return function(...arguments_) {
        const styler = createStyler(getModelAnsi(model, levelMapping[level], "bgColor", ...arguments_), ansi_styles_default.bgColor.close, this[STYLER]);
        return createBuilder(this, styler, this[IS_EMPTY]);
      };
    }
  };
}
var proto = Object.defineProperties(() => {
}, {
  ...styles2,
  level: {
    enumerable: true,
    get() {
      return this[GENERATOR].level;
    },
    set(level) {
      this[GENERATOR].level = level;
    }
  }
});
var createStyler = (open, close, parent) => {
  let openAll;
  let closeAll;
  if (parent === void 0) {
    openAll = open;
    closeAll = close;
  } else {
    openAll = parent.openAll + open;
    closeAll = close + parent.closeAll;
  }
  return {
    open,
    close,
    openAll,
    closeAll,
    parent
  };
};
var createBuilder = (self, _styler, _isEmpty) => {
  const builder = (...arguments_) => applyStyle(builder, arguments_.length === 1 ? "" + arguments_[0] : arguments_.join(" "));
  Object.setPrototypeOf(builder, proto);
  builder[GENERATOR] = self;
  builder[STYLER] = _styler;
  builder[IS_EMPTY] = _isEmpty;
  return builder;
};
var applyStyle = (self, string) => {
  if (self.level <= 0 || !string) {
    return self[IS_EMPTY] ? "" : string;
  }
  let styler = self[STYLER];
  if (styler === void 0) {
    return string;
  }
  const { openAll, closeAll } = styler;
  if (string.includes("\x1B")) {
    while (styler !== void 0) {
      string = stringReplaceAll(string, styler.close, styler.open);
      styler = styler.parent;
    }
  }
  const lfIndex = string.indexOf("\n");
  if (lfIndex !== -1) {
    string = stringEncaseCRLFWithFirstIndex(string, closeAll, openAll, lfIndex);
  }
  return openAll + string + closeAll;
};
Object.defineProperties(createChalk.prototype, styles2);
var chalk = createChalk();
var chalkStderr = createChalk({ level: stderrColor ? stderrColor.level : 0 });
var source_default = chalk;

// cli.mjs
import path from "path";
import { fileURLToPath } from "url";
import { dirname } from "path";
import Database from "better-sqlite3";
import fs from "fs";
import { execSync } from "child_process";
import os2 from "os";
var __filename = fileURLToPath(import.meta.url);
var __dirname = dirname(__filename);
var userDataPath = path.join(os2.homedir(), ".supertool");
var dbPath = path.join(userDataPath, "supertool.db");
var db;
function initDatabase() {
  try {
    if (!fs.existsSync(userDataPath)) {
      fs.mkdirSync(userDataPath, { recursive: true });
      console.log(source_default.yellow(`\u521B\u5EFA\u6570\u636E\u5E93\u76EE\u5F55: ${userDataPath}`));
    }
    db = new Database(dbPath);
    db.exec(`
      CREATE TABLE IF NOT EXISTS todos (
        id TEXT PRIMARY KEY,
        text TEXT NOT NULL,
        completed INTEGER DEFAULT 0,
        priority TEXT DEFAULT 'medium',
        dueDate TEXT,
        description TEXT,
        tag TEXT DEFAULT '',
        createdAt TEXT NOT NULL,
        updatedAt TEXT NOT NULL,
        completedAt TEXT,
        assignedTo TEXT DEFAULT '',
        assignedBy TEXT DEFAULT '',
        assignedAt TEXT,
        owner TEXT DEFAULT '',
        orderNum INTEGER DEFAULT 0,
        repeatType TEXT DEFAULT '',
        repeatInterval INTEGER DEFAULT 1,
        repeatEndDate TEXT,
        repeatCount INTEGER DEFAULT -1,
        parentTodoId TEXT,
        markdownDescription TEXT DEFAULT ''
      );

      CREATE TABLE IF NOT EXISTS tags (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT UNIQUE NOT NULL,
        createdAt TEXT NOT NULL
      );

      CREATE TABLE IF NOT EXISTS settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
      );

      CREATE TABLE IF NOT EXISTS users (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        ip TEXT NOT NULL,
        port INTEGER NOT NULL,
        lastSeen TEXT NOT NULL,
        isOnline INTEGER DEFAULT 0
      );

      CREATE TABLE IF NOT EXISTS messages (
        id TEXT PRIMARY KEY,
        fromUserId TEXT NOT NULL,
        fromUserName TEXT NOT NULL,
        toUserId TEXT NOT NULL,
        toUserName TEXT NOT NULL,
        content TEXT NOT NULL,
        type TEXT DEFAULT 'text',
        createdAt TEXT NOT NULL,
        read INTEGER DEFAULT 0
      );

      CREATE TABLE IF NOT EXISTS file_transfers (
        id TEXT PRIMARY KEY,
        fromUserId TEXT NOT NULL,
        fromUserName TEXT NOT NULL,
        toUserId TEXT NOT NULL,
        toUserName TEXT NOT NULL,
        fileName TEXT NOT NULL,
        fileSize INTEGER NOT NULL,
        filePath TEXT,
        status TEXT DEFAULT 'pending',
        progress INTEGER DEFAULT 0,
        createdAt TEXT NOT NULL
      );
      
      -- \u670D\u52A1\u5668\u7BA1\u7406\u8868
      CREATE TABLE IF NOT EXISTS servers (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        host TEXT NOT NULL,
        port INTEGER DEFAULT 22,
        username TEXT NOT NULL,
        sshKeyPath TEXT,
        password TEXT,
        description TEXT,
        tags TEXT,
        groupId TEXT,
        createdAt TEXT NOT NULL,
        updatedAt TEXT NOT NULL
      );
      
      -- CI/CD\u914D\u7F6E\u8868
      CREATE TABLE IF NOT EXISTS cicd_configs (
        id TEXT PRIMARY KEY,
        projectId TEXT NOT NULL,
        deployBranch TEXT DEFAULT 'main',
        mavenSettings TEXT,
        mavenProfile TEXT,
        sshHost TEXT,
        sshPort INTEGER DEFAULT 22,
        sshUser TEXT,
        sshKeyPath TEXT,
        sshPassword TEXT,
        deployPath TEXT,
        libSeparate INTEGER DEFAULT 1,
        restartScript TEXT,
        healthCheckUrl TEXT,
        createdAt TEXT NOT NULL,
        updatedAt TEXT NOT NULL
      );
      
      -- \u90E8\u7F72\u65E5\u5FD7\u8868
      CREATE TABLE IF NOT EXISTS deploy_logs (
        id TEXT PRIMARY KEY,
        projectId TEXT NOT NULL,
        configId TEXT NOT NULL,
        status TEXT DEFAULT 'pending',
        startTime TEXT,
        endTime TEXT,
        errorMessage TEXT,
        triggeredBy TEXT DEFAULT 'manual',
        createdAt TEXT NOT NULL
      );

      -- \u90E8\u7F72\u5386\u53F2\u8868
      CREATE TABLE IF NOT EXISTS deploy_history (
        id TEXT PRIMARY KEY,
        configId TEXT NOT NULL,
        projectId TEXT NOT NULL,
        status TEXT DEFAULT 'success',
        version TEXT,
        gitCommit TEXT,
        deployedAt TEXT,
        rolledBack INTEGER DEFAULT 0,
        rolledBackAt TEXT
      );

      -- \u90E8\u7F72\u6B65\u9AA4\u65E5\u5FD7\u8868
      CREATE TABLE IF NOT EXISTS deploy_step_logs (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        deployLogId TEXT NOT NULL,
        stepName TEXT NOT NULL,
        stepOrder INTEGER DEFAULT 0,
        status TEXT DEFAULT 'pending',
        startTime TEXT,
        endTime TEXT,
        output TEXT,
        errorMessage TEXT,
        createdAt TEXT NOT NULL
      );

      -- \u90E8\u7F72\u6A21\u5757\u8868
      CREATE TABLE IF NOT EXISTS deploy_modules (
        id TEXT PRIMARY KEY,
        configId TEXT NOT NULL,
        name TEXT NOT NULL,
        modulePath TEXT NOT NULL,
        createdAt TEXT NOT NULL,
        updatedAt TEXT NOT NULL,
        FOREIGN KEY (configId) REFERENCES cicd_configs (id) ON DELETE CASCADE
      );

      -- \u670D\u52A1\u5668\u5206\u7EC4\u8868
      CREATE TABLE IF NOT EXISTS server_groups (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT DEFAULT '',
        createdAt TEXT NOT NULL,
        updatedAt TEXT NOT NULL
      );

      -- \u5B50\u4EFB\u52A1\u8868
      CREATE TABLE IF NOT EXISTS subtasks (
        id TEXT PRIMARY KEY,
        todoId TEXT NOT NULL,
        text TEXT NOT NULL,
        completed INTEGER DEFAULT 0,
        orderNum INTEGER DEFAULT 0,
        createdAt TEXT NOT NULL,
        updatedAt TEXT NOT NULL,
        FOREIGN KEY (todoId) REFERENCES todos (id) ON DELETE CASCADE
      );
      
      -- \u9879\u76EE\u7BA1\u7406\u8868
      CREATE TABLE IF NOT EXISTS projects (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT,
        status TEXT DEFAULT 'active',
        gitUrl TEXT,
        color TEXT,
        repoPath TEXT,
        gitUrl1 TEXT,
        gitUrl2 TEXT,
        category TEXT,
        archived INTEGER,
        startDate TEXT,
        endDate TEXT,
        createdAt TEXT NOT NULL,
        updatedAt TEXT NOT NULL
      );

      -- \u5468\u62A5\u7BA1\u7406\u8868
      CREATE TABLE IF NOT EXISTS weekly_reports (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        startDate TEXT NOT NULL,
        endDate TEXT NOT NULL,
        data TEXT NOT NULL,
        createdAt TEXT NOT NULL
      );

      -- \u7B14\u8BB0\u5206\u7EC4\u8868
      CREATE TABLE IF NOT EXISTS note_groups (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL DEFAULT '',
        icon TEXT NOT NULL DEFAULT '',
        sortOrder INTEGER DEFAULT 0,
        createdAt TEXT NOT NULL
      );

      -- \u7B14\u8BB0\u8868
      CREATE TABLE IF NOT EXISTS notes (
        id TEXT PRIMARY KEY,
        title TEXT NOT NULL DEFAULT '',
        content TEXT NOT NULL DEFAULT '',
        description TEXT,
        tags TEXT NOT NULL DEFAULT '[]',
        pinned INTEGER DEFAULT 0,
        groupId TEXT,
        createdAt TEXT NOT NULL,
        updatedAt TEXT NOT NULL
      );

      -- MFA/TOTP \u5BC6\u94A5\u8868
      CREATE TABLE IF NOT EXISTS mfa_secrets (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        secret TEXT NOT NULL,
        digits INTEGER DEFAULT 6,
        period INTEGER DEFAULT 30,
        algorithm TEXT DEFAULT 'sha1',
        account TEXT DEFAULT '',
        issuer TEXT DEFAULT '',
        createdAt TEXT NOT NULL,
        updatedAt TEXT NOT NULL
      );

      -- \u8BB0\u8D26\u76F8\u5173
      CREATE TABLE IF NOT EXISTS accounting_categories (
        id TEXT PRIMARY KEY, name TEXT NOT NULL DEFAULT '', type TEXT NOT NULL DEFAULT 'expense',
        icon TEXT NOT NULL DEFAULT '', sortOrder INTEGER DEFAULT 0, createdAt TEXT NOT NULL
      );
      CREATE TABLE IF NOT EXISTS accounting_records (
        id TEXT PRIMARY KEY, date TEXT NOT NULL, type TEXT NOT NULL DEFAULT 'expense',
        category TEXT NOT NULL DEFAULT '', amount REAL NOT NULL DEFAULT 0, description TEXT DEFAULT '',
        status TEXT DEFAULT 'completed', attachmentPath TEXT, createdBy TEXT DEFAULT '',
        createdAt TEXT NOT NULL, updatedAt TEXT NOT NULL,
        voucher_number TEXT, receipt_type TEXT, receipt_path TEXT,
        entity TEXT, project TEXT, supplier TEXT, invoice_number TEXT,
        tax_amount REAL, payment_method TEXT, approver TEXT, attachments_json TEXT
      );
      CREATE TABLE IF NOT EXISTS accounting_budgets (
        id TEXT PRIMARY KEY, category TEXT NOT NULL, amount REAL NOT NULL DEFAULT 0,
        period TEXT NOT NULL DEFAULT 'monthly', createdAt TEXT NOT NULL
      );
      CREATE TABLE IF NOT EXISTS accounting_templates (
        id TEXT PRIMARY KEY, name TEXT NOT NULL, type TEXT NOT NULL DEFAULT 'expense',
        category TEXT NOT NULL, amount REAL NOT NULL DEFAULT 0, description TEXT DEFAULT '',
        entity TEXT DEFAULT '', project TEXT DEFAULT '', supplier TEXT DEFAULT '',
        payment_method TEXT DEFAULT '', tax_rate REAL DEFAULT 0,
        useCount INTEGER DEFAULT 0, createdAt TEXT NOT NULL
      );
      -- Git \u7BA1\u7406
      CREATE TABLE IF NOT EXISTS git_repos (
        id TEXT PRIMARY KEY, name TEXT NOT NULL, path TEXT NOT NULL UNIQUE,
        remote TEXT, branch TEXT, lastOpened TEXT, createdAt TEXT NOT NULL, updatedAt TEXT NOT NULL
      );
      -- \u65E5\u5FD7\u9884\u8BBE
      CREATE TABLE IF NOT EXISTS log_presets (
        id TEXT PRIMARY KEY, name TEXT NOT NULL, serverIds TEXT NOT NULL,
        logPath TEXT NOT NULL, logType TEXT NOT NULL DEFAULT 'file', keywords TEXT DEFAULT '[]',
        maxLines INTEGER DEFAULT 500, createdAt TEXT NOT NULL, updatedAt TEXT NOT NULL
      );
      -- API \u8C03\u8BD5
      CREATE TABLE IF NOT EXISTS api_requests (
        id TEXT PRIMARY KEY, name TEXT NOT NULL DEFAULT '', method TEXT NOT NULL DEFAULT 'GET',
        url TEXT NOT NULL, headers TEXT NOT NULL DEFAULT '{}', body TEXT,
        contentType TEXT DEFAULT 'none', createdAt TEXT NOT NULL, updatedAt TEXT NOT NULL
      );
      -- \u8BA1\u7B97\u5668\u5386\u53F2
      CREATE TABLE IF NOT EXISTS calculator_history (
        id TEXT PRIMARY KEY, expression TEXT NOT NULL, result TEXT NOT NULL, createdAt TEXT NOT NULL
      );
      -- OpenVPN
      CREATE TABLE IF NOT EXISTS openvpn_configs (
        id TEXT PRIMARY KEY, name TEXT NOT NULL DEFAULT '', filePath TEXT NOT NULL DEFAULT '',
        content TEXT NOT NULL, createdAt TEXT NOT NULL, updatedAt TEXT NOT NULL
      );
      -- \u804A\u5929\u6D88\u606F
      CREATE TABLE IF NOT EXISTS chat_messages (
        id TEXT PRIMARY KEY, roomId TEXT NOT NULL, fromUserId TEXT NOT NULL, fromUserName TEXT NOT NULL,
        toUserId TEXT, toUserName TEXT, content TEXT NOT NULL, type TEXT DEFAULT 'text',
        createdAt TEXT NOT NULL, read INTEGER DEFAULT 0
      );
    `);
    const defaultTags = ["\u5DE5\u4F5C", "\u751F\u6D3B", "\u5B66\u4E60", "\u5176\u4ED6"];
    const insertTag = db.prepare("INSERT OR IGNORE INTO tags (name, createdAt) VALUES (?, ?)");
    defaultTags.forEach((tag) => {
      insertTag.run(tag, (/* @__PURE__ */ new Date()).toISOString());
    });
    return db;
  } catch (error) {
    console.error(source_default.red(`\u6570\u636E\u5E93\u521D\u59CB\u5316\u9519\u8BEF: ${error.message}`));
    console.error(source_default.yellow(`\u5C1D\u8BD5\u8BBF\u95EE\u6570\u636E\u5E93: ${dbPath}`));
    process.exit(1);
  }
}
function getDatabase() {
  if (!db) {
    initDatabase();
  }
  return db;
}
function getAllTodos() {
  const stmt = getDatabase().prepare(`
    SELECT * FROM todos 
    ORDER BY 
      CASE WHEN completed = 1 THEN 1 ELSE 0 END,
      orderNum ASC, 
      createdAt DESC
  `);
  const rows = stmt.all();
  return rows.map((row) => ({
    id: row.id,
    text: row.text,
    completed: row.completed === 1,
    priority: row.priority,
    dueDate: row.dueDate,
    description: row.description,
    tag: row.tag,
    createdAt: row.createdAt,
    updatedAt: row.updatedAt,
    completedAt: row.completedAt,
    orderNum: row.orderNum,
    repeatType: row.repeatType,
    repeatInterval: row.repeatInterval,
    repeatEndDate: row.repeatEndDate,
    repeatCount: row.repeatCount,
    parentTodoId: row.parentTodoId,
    markdownDescription: row.markdownDescription
  }));
}
function addTodo(text, options = {}) {
  const id = Date.now().toString();
  const now = (/* @__PURE__ */ new Date()).toISOString();
  const stmt = getDatabase().prepare(`
    INSERT INTO todos (
      id, text, completed, priority, dueDate, description, tag, 
      createdAt, updatedAt, repeatType, repeatInterval, repeatEndDate, 
      repeatCount, parentTodoId, markdownDescription
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
  stmt.run(
    id,
    text,
    0,
    // 默认未完成
    options.priority || "medium",
    options.dueDate || null,
    options.description || "",
    options.tag || "\u672A\u5206\u7C7B",
    now,
    now,
    options.repeatType || "",
    options.repeatInterval || 1,
    options.repeatEndDate || null,
    options.repeatCount || -1,
    options.parentTodoId || null,
    options.markdownDescription || ""
  );
  return { id, text, completed: false, priority: options.priority || "medium", tag: options.tag || "\u672A\u5206\u7C7B" };
}
function updateTodo(id, updates) {
  const setClause = Object.keys(updates).map((key) => `${key} = ?`).join(", ");
  const values = Object.values(updates);
  values.push(id);
  const stmt = getDatabase().prepare(`UPDATE todos SET ${setClause} WHERE id = ?`);
  stmt.run(...values);
  return getTodoById(id);
}
function deleteTodo(id) {
  const stmt = getDatabase().prepare("DELETE FROM todos WHERE id = ?");
  stmt.run(id);
  return true;
}
function getTodoById(id) {
  const stmt = getDatabase().prepare("SELECT * FROM todos WHERE id = ?");
  const row = stmt.get(id);
  if (!row) return null;
  return {
    id: row.id,
    text: row.text,
    completed: row.completed === 1,
    priority: row.priority,
    dueDate: row.dueDate,
    description: row.description,
    tag: row.tag,
    createdAt: row.createdAt,
    updatedAt: row.updatedAt,
    completedAt: row.completedAt,
    orderNum: row.orderNum,
    repeatType: row.repeatType,
    repeatInterval: row.repeatInterval,
    repeatEndDate: row.repeatEndDate,
    repeatCount: row.repeatCount,
    parentTodoId: row.parentTodoId,
    markdownDescription: row.markdownDescription
  };
}
function formatDate(dateString) {
  if (!dateString) return "";
  const date = new Date(dateString);
  return date.toLocaleDateString("zh-CN");
}
function formatPriority(priority) {
  const colors = {
    high: source_default.red.bold,
    medium: source_default.yellow,
    low: source_default.green
  };
  const labels = {
    high: "\u{1F534} \u9AD8",
    medium: "\u{1F7E1} \u4E2D",
    low: "\u{1F7E2} \u4F4E"
  };
  const colorFn = colors[priority] || ((s) => s);
  return colorFn(labels[priority] || priority);
}
function printTasks(tasks, options = {}) {
  if (options.json) {
    console.log(JSON.stringify(tasks, null, 2));
    return;
  }
  if (tasks.length === 0) {
    console.log(source_default.gray("\u6682\u65E0\u4EFB\u52A1"));
    return;
  }
  tasks.forEach((task) => {
    const status = task.completed ? source_default.green("\u2713") : source_default.red("\u25CB");
    const priority = formatPriority(task.priority);
    const dueDate = task.dueDate ? source_default.blue(` [${formatDate(task.dueDate)}]`) : "";
    const tag = task.tag ? source_default.magenta(` #${task.tag}`) : "";
    const textDecoration = task.completed ? source_default.strikethrough : (s) => s;
    console.log(`${status} ${textDecoration(task.text)} ${priority}${dueDate}${tag}`);
  });
}
function printTaskDetails(task, options = {}) {
  if (options.json) {
    console.log(JSON.stringify(task, null, 2));
    return;
  }
  if (!task) {
    console.log(source_default.red("\u4EFB\u52A1\u4E0D\u5B58\u5728"));
    return;
  }
  console.log(source_default.bold(task.text));
  console.log("-".repeat(task.text.length || 20));
  console.log(`${source_default.bold("ID:")} ${task.id}`);
  console.log(`${source_default.bold("\u72B6\u6001:")} ${task.completed ? source_default.green("\u5DF2\u5B8C\u6210") : source_default.red("\u5F85\u529E")}`);
  console.log(`${source_default.bold("\u4F18\u5148\u7EA7:")} ${formatPriority(task.priority)}`);
  console.log(`${source_default.bold("\u6807\u7B7E:")} ${source_default.magenta(task.tag)}`);
  console.log(`${source_default.bold("\u521B\u5EFA\u65F6\u95F4:")} ${formatDate(task.createdAt)}`);
  if (task.dueDate) {
    console.log(`${source_default.bold("\u622A\u6B62\u65E5\u671F:")} ${source_default.blue(formatDate(task.dueDate))}`);
  }
  if (task.completedAt) {
    console.log(`${source_default.bold("\u5B8C\u6210\u65F6\u95F4:")} ${formatDate(task.completedAt)}`);
  }
  if (task.description) {
    console.log(`${source_default.bold("\u63CF\u8FF0:")} ${task.description}`);
  }
  if (task.markdownDescription) {
    console.log(`${source_default.bold("\u8BE6\u7EC6\u5185\u5BB9:")} ${task.markdownDescription.substring(0, 50)}...`);
  }
}
function printStats(stats, options = {}) {
  if (options.json) {
    console.log(JSON.stringify(stats, null, 2));
    return;
  }
  console.log(source_default.bold("\u4EFB\u52A1\u7EDF\u8BA1"));
  console.log("-".repeat(10));
  console.log(`${source_default.bold("\u603B\u8BA1:")} ${stats.total}`);
  console.log(`${source_default.bold("\u5F85\u529E:")} ${source_default.red(stats.active)}`);
  console.log(`${source_default.bold("\u5DF2\u5B8C\u6210:")} ${source_default.green(stats.completed)}`);
  console.log(`${source_default.bold("\u5B8C\u6210\u7387:")} ${stats.total > 0 ? (stats.completed / stats.total * 100).toFixed(1) : 0}%`);
}
program.name("stool").description("SuperTool CLI - \u5F00\u53D1\u8005\u5168\u80FD\u5DE5\u5177\u7BB1").version("1.0.0");
program.command("add").alias("a").description("\u6DFB\u52A0\u65B0\u4EFB\u52A1").argument("<text>", "\u4EFB\u52A1\u5185\u5BB9").option("-p, --priority <level>", "\u4F18\u5148\u7EA7 (high|medium|low)", "medium").option("-d, --due-date <date>", "\u622A\u6B62\u65E5\u671F (YYYY-MM-DD)").option("-t, --tag <tag>", "\u6807\u7B7E").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action(async (text, options) => {
  try {
    const newTask = addTodo(text, {
      priority: options.priority,
      dueDate: options["dueDate"],
      tag: options.tag
    });
    if (options.json) {
      console.log(JSON.stringify(newTask, null, 2));
    } else {
      console.log(source_default.green(`\u2713 \u4EFB\u52A1\u5DF2\u6DFB\u52A0: ${newTask.text}`));
    }
  } catch (error) {
    console.error(source_default.red(`\u6DFB\u52A0\u4EFB\u52A1\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
program.command("list").alias("ls").description("\u5217\u51FA\u4EFB\u52A1").option("-a, --all", "\u663E\u793A\u6240\u6709\u4EFB\u52A1 (\u9ED8\u8BA4\u53EA\u663E\u793A\u5F85\u529E)").option("-c, --completed", "\u53EA\u663E\u793A\u5DF2\u5B8C\u6210\u4EFB\u52A1").option("-t, --tag <tag>", "\u6309\u6807\u7B7E\u7B5B\u9009").option("-p, --priority <level>", "\u6309\u4F18\u5148\u7EA7\u7B5B\u9009").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((options) => {
  try {
    let tasks = getAllTodos();
    if (!options.all && !options.completed) {
      tasks = tasks.filter((task) => !task.completed);
    } else if (options.completed) {
      tasks = tasks.filter((task) => task.completed);
    }
    if (options.tag) {
      tasks = tasks.filter((task) => task.tag === options.tag);
    }
    if (options.priority) {
      tasks = tasks.filter((task) => task.priority === options.priority);
    }
    printTasks(tasks, options);
  } catch (error) {
    console.error(source_default.red(`\u5217\u51FA\u4EFB\u52A1\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
program.command("complete").alias("done").description("\u6807\u8BB0\u4EFB\u52A1\u4E3A\u5B8C\u6210").argument("<id>", "\u4EFB\u52A1ID").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((id, options) => {
  try {
    const task = getTodoById(id);
    if (!task) {
      if (options.json) {
        console.log(JSON.stringify({ error: "\u4EFB\u52A1\u4E0D\u5B58\u5728", id }, null, 2));
      } else {
        console.log(source_default.red(`\u4EFB\u52A1\u4E0D\u5B58\u5728: ${id}`));
      }
      return;
    }
    const updatedTask = updateTodo(id, {
      completed: 1,
      completedAt: (/* @__PURE__ */ new Date()).toISOString(),
      updatedAt: (/* @__PURE__ */ new Date()).toISOString()
    });
    if (options.json) {
      console.log(JSON.stringify(updatedTask, null, 2));
    } else {
      console.log(source_default.green(`\u2713 \u4EFB\u52A1\u5DF2\u5B8C\u6210: ${updatedTask.text}`));
    }
  } catch (error) {
    console.error(source_default.red(`\u5B8C\u6210\u4EFB\u52A1\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
program.command("delete").alias("del").description("\u5220\u9664\u4EFB\u52A1").argument("<id>", "\u4EFB\u52A1ID").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((id, options) => {
  try {
    const task = getTodoById(id);
    if (!task) {
      if (options.json) {
        console.log(JSON.stringify({ error: "\u4EFB\u52A1\u4E0D\u5B58\u5728", id }, null, 2));
      } else {
        console.log(source_default.red(`\u4EFB\u52A1\u4E0D\u5B58\u5728: ${id}`));
      }
      return;
    }
    deleteTodo(id);
    if (options.json) {
      console.log(JSON.stringify({ success: true, id }, null, 2));
    } else {
      console.log(source_default.green(`\u2713 \u4EFB\u52A1\u5DF2\u5220\u9664: ${task.text}`));
    }
  } catch (error) {
    console.error(source_default.red(`\u5220\u9664\u4EFB\u52A1\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
program.command("show").alias("view").description("\u663E\u793A\u4EFB\u52A1\u8BE6\u60C5").argument("<id>", "\u4EFB\u52A1ID").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((id, options) => {
  try {
    const task = getTodoById(id);
    printTaskDetails(task, options);
  } catch (error) {
    console.error(source_default.red(`\u663E\u793A\u4EFB\u52A1\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
program.command("stats").alias("stat").description("\u663E\u793A\u4EFB\u52A1\u7EDF\u8BA1\u4FE1\u606F").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((options) => {
  try {
    const tasks = getAllTodos();
    const stats = {
      total: tasks.length,
      active: tasks.filter((t2) => !t2.completed).length,
      completed: tasks.filter((t2) => t2.completed).length
    };
    printStats(stats, options);
  } catch (error) {
    console.error(source_default.red(`\u83B7\u53D6\u7EDF\u8BA1\u4FE1\u606F\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
program.command("clear").alias("clean").description("\u6E05\u7A7A\u5DF2\u5B8C\u6210\u7684\u4EFB\u52A1").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((options) => {
  try {
    const completedTasks = getAllTodos().filter((t2) => t2.completed);
    const count = completedTasks.length;
    completedTasks.forEach((task) => deleteTodo(task.id));
    if (options.json) {
      console.log(JSON.stringify({ success: true, count }, null, 2));
    } else {
      console.log(source_default.green(`\u2713 \u5DF2\u6E05\u7A7A ${count} \u4E2A\u5DF2\u5B8C\u6210\u4EFB\u52A1`));
    }
  } catch (error) {
    console.error(source_default.red(`\u6E05\u7A7A\u4EFB\u52A1\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
program.command("send-message").description("\u53D1\u9001\u6D88\u606F\u7ED9\u5C40\u57DF\u7F51\u7528\u6237").argument("<message>", "\u6D88\u606F\u5185\u5BB9").option("-t, --to <userId>", "\u63A5\u6536\u8005ID (\u7559\u7A7A\u5219\u5E7F\u64AD)").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action(async (message, options) => {
  try {
    const stmt = getDatabase().prepare("SELECT * FROM users WHERE isOnline = 1");
    const peers = stmt.all();
    if (peers.length === 0) {
      console.log(source_default.yellow("\u6CA1\u6709\u5728\u7EBF\u7528\u6237"));
      return;
    }
    let targetPeer = null;
    if (options.to) {
      targetPeer = peers.find((p2) => p2.id === options.to);
      if (!targetPeer) {
        console.log(source_default.red(`\u7528\u6237\u4E0D\u5B58\u5728: ${options.to}`));
        return;
      }
    }
    const messageId = Date.now().toString() + Math.random().toString(36).substr(2, 9);
    const stmtInsert = getDatabase().prepare(`
        INSERT INTO messages (id, fromUserId, fromUserName, toUserId, toUserName, content, type, createdAt)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
      `);
    if (targetPeer) {
      stmtInsert.run(
        messageId,
        "cli-user",
        // CLI工具的虚拟用户ID
        "CLI-User",
        targetPeer.id,
        targetPeer.name,
        message,
        "text",
        (/* @__PURE__ */ new Date()).toISOString()
      );
      if (options.json) {
        console.log(JSON.stringify({ success: true, message: "\u6D88\u606F\u5DF2\u53D1\u9001", to: targetPeer.name }, null, 2));
      } else {
        console.log(source_default.green(`\u2713 \u6D88\u606F\u5DF2\u53D1\u9001\u7ED9 ${targetPeer.name}`));
      }
    } else {
      for (const peer of peers) {
        stmtInsert.run(
          Date.now().toString() + Math.random().toString(36).substr(2, 9),
          "cli-user",
          "CLI-User",
          peer.id,
          peer.name,
          message,
          "text",
          (/* @__PURE__ */ new Date()).toISOString()
        );
      }
      if (options.json) {
        console.log(JSON.stringify({ success: true, message: "\u6D88\u606F\u5DF2\u5E7F\u64AD", count: peers.length }, null, 2));
      } else {
        console.log(source_default.green(`\u2713 \u6D88\u606F\u5DF2\u5E7F\u64AD\u7ED9 ${peers.length} \u4E2A\u5728\u7EBF\u7528\u6237`));
      }
    }
  } catch (error) {
    console.error(source_default.red(`\u53D1\u9001\u6D88\u606F\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
program.command("send-file").description("\u53D1\u9001\u6587\u4EF6\u7ED9\u5C40\u57DF\u7F51\u7528\u6237").argument("<file-path>", "\u6587\u4EF6\u8DEF\u5F84").argument("<user-id>", "\u63A5\u6536\u8005ID").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action(async (filePath, userId, options) => {
  try {
    if (!fs.existsSync(filePath)) {
      console.log(source_default.red(`\u6587\u4EF6\u4E0D\u5B58\u5728: ${filePath}`));
      return;
    }
    const stmt = getDatabase().prepare("SELECT * FROM users WHERE id = ?");
    const targetPeer = stmt.get(userId);
    if (!targetPeer) {
      console.log(source_default.red(`\u7528\u6237\u4E0D\u5B58\u5728: ${userId}`));
      return;
    }
    const stats = fs.statSync(filePath);
    const fileName = path.basename(filePath);
    const transferId = Date.now().toString() + Math.random().toString(36).substr(2, 9);
    const stmtInsert = getDatabase().prepare(`
        INSERT INTO file_transfers (id, fromUserId, fromUserName, toUserId, toUserName, fileName, fileSize, filePath, status, progress, createdAt)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      `);
    stmtInsert.run(
      transferId,
      "cli-user",
      "CLI-User",
      targetPeer.id,
      targetPeer.name,
      fileName,
      stats.size,
      filePath,
      "pending",
      0,
      (/* @__PURE__ */ new Date()).toISOString()
    );
    if (options.json) {
      console.log(JSON.stringify({
        success: true,
        message: "\u6587\u4EF6\u4F20\u8F93\u5DF2\u5F00\u59CB",
        transferId,
        fileName,
        to: targetPeer.name
      }, null, 2));
    } else {
      console.log(source_default.green(`\u2713 \u6587\u4EF6\u4F20\u8F93\u5DF2\u5F00\u59CB: ${fileName} -> ${targetPeer.name}`));
    }
    console.log(source_default.yellow("\u6CE8\u610F: \u5B9E\u9645\u6587\u4EF6\u4F20\u8F93\u9700\u8981\u901A\u8FC7GUI\u5E94\u7528\u5B8C\u6210"));
  } catch (error) {
    console.error(source_default.red(`\u53D1\u9001\u6587\u4EF6\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
program.command("messages").alias("msgs").description("\u67E5\u770B\u6D88\u606F\u5386\u53F2").option("-l, --limit <limit>", "\u9650\u5236\u6570\u91CF", "20").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action(async (options) => {
  try {
    const limit = parseInt(options.limit);
    const stmt = getDatabase().prepare(`
        SELECT * FROM messages 
        ORDER BY createdAt DESC 
        LIMIT ?
      `);
    const messages = stmt.all(limit);
    if (options.json) {
      console.log(JSON.stringify(messages, null, 2));
      return;
    }
    if (messages.length === 0) {
      console.log(source_default.gray("\u6682\u65E0\u6D88\u606F"));
      return;
    }
    console.log(source_default.bold("\u6D88\u606F\u5386\u53F2:"));
    console.log("-".repeat(50));
    messages.reverse().forEach((msg) => {
      const date = new Date(msg.createdAt);
      const timeStr = date.toLocaleTimeString("zh-CN");
      const dateStr = date.toLocaleDateString("zh-CN");
      console.log(`${source_default.blue(`${msg.fromUserName}`)} -> ${source_default.green(`${msg.toUserName}`)} [${dateStr} ${timeStr}]`);
      console.log(`${msg.content}`);
      console.log("-".repeat(50));
    });
  } catch (error) {
    console.error(source_default.red(`\u83B7\u53D6\u6D88\u606F\u5386\u53F2\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
program.command("peers").description("\u67E5\u770B\u5C40\u57DF\u7F51\u53D1\u73B0\u7684\u7528\u6237").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action(async (options) => {
  try {
    const stmt = getDatabase().prepare(`
        SELECT id, name, ip, port, lastSeen, isOnline 
        FROM users 
        ORDER BY lastSeen DESC
      `);
    const peers = stmt.all();
    if (options.json) {
      console.log(JSON.stringify(peers, null, 2));
      return;
    }
    if (peers.length === 0) {
      console.log(source_default.gray("\u6682\u65E0\u5C40\u57DF\u7F51\u7528\u6237"));
      return;
    }
    console.log(source_default.bold("\u5C40\u57DF\u7F51\u7528\u6237\u5217\u8868:"));
    console.log("-".repeat(80));
    console.log(`${source_default.bold("\u7528\u6237ID".padEnd(20))} ${source_default.bold("\u7528\u6237\u540D".padEnd(15))} ${source_default.bold("IP\u5730\u5740".padEnd(15))} ${source_default.bold("\u7AEF\u53E3".padEnd(8))} ${source_default.bold("\u6700\u540E\u6D3B\u52A8".padEnd(15))} ${source_default.bold("\u5728\u7EBF\u72B6\u6001")}`);
    console.log("-".repeat(80));
    peers.forEach((peer) => {
      const date = new Date(peer.lastSeen);
      const timeStr = date.toLocaleDateString("zh-CN");
      const status = peer.isOnline ? source_default.green("\u5728\u7EBF") : source_default.red("\u79BB\u7EBF");
      console.log(`${peer.id.substring(0, 18).padEnd(20)} ${peer.name.padEnd(15)} ${peer.ip.padEnd(15)} ${peer.port.toString().padEnd(8)} ${timeStr.padEnd(15)} ${status}`);
    });
  } catch (error) {
    console.error(source_default.red(`\u83B7\u53D6\u7528\u6237\u5217\u8868\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
program.command("help-detail").alias("help").description("\u663E\u793A\u8BE6\u7EC6\u5E2E\u52A9\u4FE1\u606F").action(() => {
  printFullHelp();
});
var serverCmd = program.command("server").description("\u670D\u52A1\u5668\u7BA1\u7406");
serverCmd.command("list").alias("ls").description("\u67E5\u770B\u6240\u6709\u670D\u52A1\u5668").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((options) => {
  try {
    const stmt = getDatabase().prepare("SELECT * FROM servers ORDER BY createdAt DESC");
    const servers = stmt.all();
    if (options.json) {
      console.log(JSON.stringify(servers.map((s) => ({
        id: s.id,
        name: s.name,
        host: s.host,
        port: s.port,
        username: s.username,
        tags: s.tags ? s.tags.split(",") : [],
        description: s.description,
        createdAt: s.createdAt
      })), null, 2));
      return;
    }
    if (servers.length === 0) {
      console.log(source_default.gray("\u6682\u65E0\u670D\u52A1\u5668"));
      return;
    }
    console.log(source_default.bold("\n\u670D\u52A1\u5668\u5217\u8868:\n"));
    servers.forEach((server) => {
      const tags = server.tags ? server.tags.split(",") : [];
      console.log(source_default.cyan(`  ${server.name}`) + source_default.gray(` (${server.host}:${server.port})`));
      console.log(source_default.gray(`    ID: ${server.id}`));
      console.log(source_default.gray(`    \u7528\u6237: ${server.username}`));
      if (tags.length > 0) {
        console.log(source_default.gray(`    \u6807\u7B7E: ${tags.join(", ")}`));
      }
      console.log("");
    });
  } catch (error) {
    console.error(source_default.red(`\u83B7\u53D6\u670D\u52A1\u5668\u5217\u8868\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
serverCmd.command("add").description("\u6DFB\u52A0\u670D\u52A1\u5668").argument("<name>", "\u670D\u52A1\u5668\u540D\u79F0").argument("<host>", "\u4E3B\u673A\u5730\u5740").option("-p, --port <port>", "SSH\u7AEF\u53E3", "22").option("-u, --user <user>", "\u7528\u6237\u540D", "root").option("-k, --key <path>", "SSH Key\u8DEF\u5F84").option("-w, --password <pwd>", "\u5BC6\u7801").option("-t, --tags <tags>", "\u6807\u7B7E (\u9017\u53F7\u5206\u9694)").option("-d, --description <desc>", "\u63CF\u8FF0").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((name, host, options) => {
  try {
    const id = Date.now().toString();
    const now = (/* @__PURE__ */ new Date()).toISOString();
    const stmt = getDatabase().prepare(`
        INSERT INTO servers (id, name, host, port, username, sshKeyPath, password, description, tags, createdAt, updatedAt)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      `);
    stmt.run(
      id,
      name,
      host,
      parseInt(options.port) || 22,
      options.user || "root",
      options.key || null,
      options.password || null,
      options.description || "",
      options.tags || "",
      now,
      now
    );
    if (options.json) {
      console.log(JSON.stringify({ success: true, id, name, host }, null, 2));
    } else {
      console.log(source_default.green(`\u2713 \u670D\u52A1\u5668\u5DF2\u6DFB\u52A0: ${name} (${host})`));
      console.log(source_default.gray(`  ID: ${id}`));
    }
  } catch (error) {
    console.error(source_default.red(`\u6DFB\u52A0\u670D\u52A1\u5668\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
serverCmd.command("delete").alias("rm").description("\u5220\u9664\u670D\u52A1\u5668").argument("<id>", "\u670D\u52A1\u5668ID").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((id, options) => {
  try {
    const stmt = getDatabase().prepare("DELETE FROM servers WHERE id = ?");
    stmt.run(id);
    if (options.json) {
      console.log(JSON.stringify({ success: true, id }, null, 2));
    } else {
      console.log(source_default.green(`\u2713 \u670D\u52A1\u5668\u5DF2\u5220\u9664: ${id}`));
    }
  } catch (error) {
    console.error(source_default.red(`\u5220\u9664\u670D\u52A1\u5668\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
serverCmd.command("test").description("\u6D4B\u8BD5\u670D\u52A1\u5668\u8FDE\u63A5").argument("<id>", "\u670D\u52A1\u5668ID").action(async (id) => {
  try {
    const stmt = getDatabase().prepare("SELECT * FROM servers WHERE id = ?");
    const server = stmt.get(id);
    if (!server) {
      console.log(source_default.red(`\u670D\u52A1\u5668\u4E0D\u5B58\u5728: ${id}`));
      return;
    }
    console.log(source_default.yellow(`\u6B63\u5728\u6D4B\u8BD5\u8FDE\u63A5: ${server.name} (${server.host}:${server.port})...`));
    const { Client } = await import("ssh2");
    const conn = new Client();
    conn.on("ready", () => {
      console.log(source_default.green(`\u2713 \u8FDE\u63A5\u6210\u529F!`));
      conn.end();
    });
    conn.on("error", (err) => {
      console.log(source_default.red(`\u2717 \u8FDE\u63A5\u5931\u8D25: ${err.message}`));
    });
    const config = {
      host: server.host,
      port: server.port,
      username: server.username,
      readyTimeout: 1e4
    };
    if (server.sshKeyPath) {
      config.privateKey = fs.readFileSync(server.sshKeyPath);
    } else if (server.password) {
      config.password = server.password;
    }
    conn.connect(config);
  } catch (error) {
    console.error(source_default.red(`\u6D4B\u8BD5\u8FDE\u63A5\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
var cicdCmd = program.command("cicd").description("CI/CD\u7BA1\u7406");
cicdCmd.command("config").description("\u914D\u7F6ECI/CD").argument("<projectId>", "\u9879\u76EEID").option("-b, --branch <branch>", "\u90E8\u7F72\u5206\u652F", "main").option("-p, --path <path>", "\u90E8\u7F72\u8DEF\u5F84").option("-s, --server <id>", "\u670D\u52A1\u5668ID").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((projectId, options) => {
  try {
    let server = null;
    if (options.server) {
      const stmt2 = getDatabase().prepare("SELECT * FROM servers WHERE id = ?");
      server = stmt2.get(options.server);
      if (!server) {
        console.log(source_default.red(`\u670D\u52A1\u5668\u4E0D\u5B58\u5728: ${options.server}`));
        return;
      }
    }
    const id = Date.now().toString();
    const now = (/* @__PURE__ */ new Date()).toISOString();
    const stmt = getDatabase().prepare(`
        INSERT INTO cicd_configs (id, projectId, deployBranch, deployPath, sshHost, sshPort, sshUser, sshKeyPath, sshPassword, createdAt, updatedAt)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      `);
    stmt.run(
      id,
      projectId,
      options.branch || "main",
      options.path || "/opt/app",
      server?.host || "",
      server?.port || 22,
      server?.username || "root",
      server?.sshKeyPath || null,
      server?.password || null,
      now,
      now
    );
    if (options.json) {
      console.log(JSON.stringify({ success: true, id, projectId, server: options.server }, null, 2));
    } else {
      console.log(source_default.green(`\u2713 CI/CD\u914D\u7F6E\u5DF2\u521B\u5EFA`));
      console.log(source_default.gray(`  \u914D\u7F6EID: ${id}`));
      console.log(source_default.gray(`  \u9879\u76EEID: ${projectId}`));
      console.log(source_default.gray(`  \u90E8\u7F72\u5206\u652F: ${options.branch || "main"}`));
      if (server) {
        console.log(source_default.gray(`  \u670D\u52A1\u5668: ${server.name} (${server.host})`));
      }
    }
  } catch (error) {
    console.error(source_default.red(`\u914D\u7F6ECI/CD\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
cicdCmd.command("logs").description("\u67E5\u770B\u90E8\u7F72\u65E5\u5FD7").argument("<projectId>", "\u9879\u76EEID").option("-l, --limit <limit>", "\u9650\u5236\u6570\u91CF", "10").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((projectId, options) => {
  try {
    const limit = parseInt(options.limit) || 10;
    const stmt = getDatabase().prepare("SELECT * FROM deploy_logs WHERE projectId = ? ORDER BY createdAt DESC LIMIT ?");
    const logs = stmt.all(projectId, limit);
    if (options.json) {
      console.log(JSON.stringify(logs, null, 2));
      return;
    }
    if (logs.length === 0) {
      console.log(source_default.gray("\u6682\u65E0\u90E8\u7F72\u65E5\u5FD7"));
      return;
    }
    console.log(source_default.bold("\n\u90E8\u7F72\u65E5\u5FD7:\n"));
    logs.forEach((log) => {
      const statusIcon = log.status === "success" ? source_default.green("\u2713") : log.status === "failed" ? source_default.red("\u2717") : source_default.yellow("\u23F3");
      console.log(`${statusIcon} ${log.createdAt} - ${log.status}`);
      if (log.errorMessage) {
        console.log(source_default.red(`    \u9519\u8BEF: ${log.errorMessage}`));
      }
    });
  } catch (error) {
    console.error(source_default.red(`\u83B7\u53D6\u90E8\u7F72\u65E5\u5FD7\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
cicdCmd.command("deploy").description("\u6267\u884C\u90E8\u7F72").argument("<projectId>", "\u9879\u76EEID").option("-d, --dry-run", "\u6A21\u62DF\u90E8\u7F72\uFF08\u4E0D\u5B9E\u9645\u6267\u884C\uFF09").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action(async (projectId, options) => {
  try {
    const configStmt = getDatabase().prepare("SELECT * FROM cicd_configs WHERE projectId = ?");
    const config = configStmt.get(projectId);
    if (!config) {
      console.log(source_default.red(`\u9879\u76EE ${projectId} \u6CA1\u6709 CI/CD \u914D\u7F6E`));
      console.log(source_default.yellow(`\u8BF7\u5148\u8FD0\u884C: todo cicd config ${projectId}`));
      process.exit(1);
    }
    let server = null;
    if (config.sshHost) {
      server = {
        host: config.sshHost,
        port: config.sshPort || 22,
        username: config.sshUser || "root",
        sshKeyPath: config.sshKeyPath,
        password: config.sshPassword
      };
    } else {
      console.log(source_default.red("CI/CD \u914D\u7F6E\u4E2D\u6CA1\u6709\u670D\u52A1\u5668\u4FE1\u606F"));
      process.exit(1);
    }
    console.log(source_default.bold("\n\u{1F680} \u5F00\u59CB\u90E8\u7F72..."));
    console.log(source_default.gray(`\u9879\u76EE: ${projectId}`));
    console.log(source_default.gray(`\u670D\u52A1\u5668: ${server.host}:${server.port}`));
    console.log(source_default.gray(`\u5206\u652F: ${config.deployBranch}`));
    console.log(source_default.gray(`\u8DEF\u5F84: ${config.deployPath}`));
    console.log("");
    if (options.dryRun) {
      console.log(source_default.yellow("\u26A0\uFE0F  \u6A21\u62DF\u90E8\u7F72\u6A21\u5F0F - \u4E0D\u5B9E\u9645\u6267\u884C"));
      console.log("");
      console.log("\u90E8\u7F72\u6B65\u9AA4:");
      console.log(source_default.gray("  1. SSH \u8FDE\u63A5\u5230\u670D\u52A1\u5668"));
      console.log(source_default.gray("  2. \u62C9\u53D6\u6700\u65B0\u4EE3\u7801 (git pull)"));
      console.log(source_default.gray("  3. Maven \u6784\u5EFA (mvn clean package -DskipTests)"));
      console.log(source_default.gray("  4. \u4F20\u8F93\u6587\u4EF6\u5230\u670D\u52A1\u5668"));
      console.log(source_default.gray("  5. \u6267\u884C\u91CD\u542F\u811A\u672C"));
      console.log(source_default.gray("  6. \u5065\u5EB7\u68C0\u67E5"));
      const logId2 = Date.now().toString();
      const now = (/* @__PURE__ */ new Date()).toISOString();
      const logStmt = getDatabase().prepare(`
          INSERT INTO deploy_logs (id, projectId, configId, status, startTime, endTime, triggeredBy, createdAt)
          VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        `);
      logStmt.run(logId2, projectId, config.id, "dry-run", now, now, "manual", now);
      console.log(source_default.green("\n\u2713 \u6A21\u62DF\u90E8\u7F72\u5B8C\u6210"));
      if (options.json) {
        console.log(JSON.stringify({ success: true, logId: logId2, status: "dry-run" }, null, 2));
      }
      return;
    }
    const logId = Date.now().toString();
    const startTime = (/* @__PURE__ */ new Date()).toISOString();
    const pendingStmt = getDatabase().prepare(`
        INSERT INTO deploy_logs (id, projectId, configId, status, startTime, triggeredBy, createdAt)
        VALUES (?, ?, ?, ?, ?, ?, ?)
      `);
    pendingStmt.run(logId, projectId, config.id, "pending", startTime, "manual", startTime);
    const { Client } = await import("ssh2");
    const conn = new Client();
    const sshConfig = {
      host: server.host,
      port: server.port,
      username: server.username,
      readyTimeout: 3e4
    };
    if (server.sshKeyPath) {
      sshConfig.privateKey = fs.readFileSync(server.sshKeyPath);
    } else if (server.password) {
      sshConfig.password = server.password;
    } else {
      console.log(source_default.red("\u6CA1\u6709 SSH \u5BC6\u94A5\u6216\u5BC6\u7801\uFF0C\u65E0\u6CD5\u8FDE\u63A5"));
      process.exit(1);
    }
    conn.on("ready", () => {
      console.log(source_default.green("\u2713 SSH \u8FDE\u63A5\u6210\u529F"));
      const deploySteps = [
        { name: "\u68C0\u67E5\u90E8\u7F72\u76EE\u5F55", cmd: `ls -la ${config.deployPath}` },
        { name: "\u62C9\u53D6\u4EE3\u7801", cmd: `cd ${config.deployPath} && git pull origin ${config.deployBranch}` },
        { name: "Maven \u6784\u5EFA", cmd: `cd ${config.deployPath} && mvn clean package -DskipTests` }
      ];
      let currentStep = 0;
      let hasError = false;
      const runNextStep = () => {
        if (currentStep >= deploySteps.length || hasError) {
          const endTime = (/* @__PURE__ */ new Date()).toISOString();
          const finalStatus = hasError ? "failed" : "success";
          const updateStmt = getDatabase().prepare(`
              UPDATE deploy_logs SET status = ?, endTime = ?, errorMessage = ? WHERE id = ?
            `);
          updateStmt.run(finalStatus, endTime, hasError ? "\u90E8\u7F72\u6B65\u9AA4\u5931\u8D25" : null, logId);
          conn.end();
          if (hasError) {
            console.log(source_default.red("\n\u2717 \u90E8\u7F72\u5931\u8D25"));
          } else {
            console.log(source_default.green("\n\u2713 \u90E8\u7F72\u5B8C\u6210"));
            if (config.restartScript) {
              console.log(source_default.gray(`\u6267\u884C\u91CD\u542F\u811A\u672C: ${config.restartScript}`));
            }
            if (config.healthCheckUrl) {
              console.log(source_default.gray(`\u5065\u5EB7\u68C0\u67E5: ${config.healthCheckUrl}`));
            }
          }
          if (options.json) {
            console.log(JSON.stringify({
              success: !hasError,
              logId,
              status: finalStatus,
              startTime,
              endTime
            }, null, 2));
          }
          return;
        }
        const step = deploySteps[currentStep];
        console.log(source_default.yellow(`
[${currentStep + 1}/${deploySteps.length}] ${step.name}`));
        console.log(source_default.gray(`  \u6267\u884C: ${step.cmd}`));
        conn.exec(step.cmd, (err, stream) => {
          if (err) {
            console.log(source_default.red(`  \u2717 \u6B65\u9AA4\u5931\u8D25: ${err.message}`));
            hasError = true;
            runNextStep();
            return;
          }
          let output = "";
          stream.on("data", (data) => {
            output += data.toString();
            const lines = data.toString().split("\n").filter((l) => l.trim());
            lines.forEach((line) => console.log(source_default.gray(`    ${line}`)));
          }).stderr.on("data", (data) => {
            output += data.toString();
            const lines = data.toString().split("\n").filter((l) => l.trim());
            lines.forEach((line) => console.log(source_default.red(`    ${line}`)));
          }).on("close", () => {
            console.log(source_default.green(`  \u2713 \u6B65\u9AA4\u5B8C\u6210`));
            currentStep++;
            runNextStep();
          });
        });
      };
      runNextStep();
    }).on("error", (err) => {
      console.log(source_default.red(`SSH \u8FDE\u63A5\u5931\u8D25: ${err.message}`));
      const endTime = (/* @__PURE__ */ new Date()).toISOString();
      const updateStmt = getDatabase().prepare(`
          UPDATE deploy_logs SET status = ?, endTime = ?, errorMessage = ? WHERE id = ?
        `);
      updateStmt.run("failed", endTime, `SSH \u8FDE\u63A5\u5931\u8D25: ${err.message}`, logId);
      if (options.json) {
        console.log(JSON.stringify({ success: false, error: err.message, logId }, null, 2));
      }
      process.exit(1);
    }).connect(sshConfig);
  } catch (error) {
    console.error(source_default.red(`\u90E8\u7F72\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
program.command("edit").alias("e").description("\u7F16\u8F91\u4EFB\u52A1").argument("<id>", "\u4EFB\u52A1ID").option("-t, --text <text>", "\u4EFB\u52A1\u6587\u672C").option("-p, --priority <priority>", "\u4F18\u5148\u7EA7 (high|medium|low)").option("-d, --due-date <date>", "\u622A\u6B62\u65E5\u671F").option("-T, --tag <tag>", "\u6807\u7B7E").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((id, options) => {
  try {
    const now = (/* @__PURE__ */ new Date()).toISOString();
    const stmt = getDatabase().prepare(`
        UPDATE todos SET 
          text = COALESCE(?, text),
          priority = COALESCE(?, priority),
          dueDate = COALESCE(?, dueDate),
          tag = COALESCE(?, tag),
          updatedAt = ?
        WHERE id = ?
      `);
    stmt.run(options.text || null, options.priority || null, options.dueDate || null, options.tag || null, now, id);
    if (options.json) {
      console.log(JSON.stringify({ success: true, id }, null, 2));
    } else {
      console.log(source_default.green("\u2713 \u4EFB\u52A1\u5DF2\u66F4\u65B0"));
    }
  } catch (error) {
    console.error(source_default.red(`\u7F16\u8F91\u5931\u8D25: ${error.message}`));
  }
});
program.command("uncomplete").alias("undo").description("\u53D6\u6D88\u5B8C\u6210\u72B6\u6001").argument("<id>", "\u4EFB\u52A1ID").action((id) => {
  try {
    const now = (/* @__PURE__ */ new Date()).toISOString();
    getDatabase().prepare("UPDATE todos SET completed = 0, completedAt = NULL, updatedAt = ? WHERE id = ?").run(now, id);
    console.log(source_default.green("\u2713 \u5DF2\u53D6\u6D88\u5B8C\u6210"));
  } catch (error) {
    console.error(source_default.red(`\u53D6\u6D88\u5931\u8D25: ${error.message}`));
  }
});
program.command("search").alias("find").description("\u641C\u7D22\u4EFB\u52A1").argument("<keyword>", "\u5173\u952E\u8BCD").option("-j, --json", "JSON\u683C\u5F0F").action((keyword, options) => {
  try {
    const stmt = getDatabase().prepare("SELECT * FROM todos WHERE text LIKE ? OR tag LIKE ? ORDER BY createdAt DESC");
    const tasks = stmt.all(`%${keyword}%`, `%${keyword}%`);
    if (options.json) {
      console.log(JSON.stringify(tasks.map((t2) => ({
        id: t2.id,
        text: t2.text,
        completed: t2.completed === 1,
        priority: t2.priority,
        tag: t2.tag,
        dueDate: t2.dueDate
      })), null, 2));
    } else {
      tasks.forEach((t2) => {
        const status = t2.completed === 1 ? "\u2713" : "\u25CB";
        console.log(`${status} ${t2.id} ${t2.text}`);
      });
    }
  } catch (error) {
    console.error(source_default.red(`\u641C\u7D22\u5931\u8D25: ${error.message}`));
  }
});
var tagCmd = program.command("tag").description("\u6807\u7B7E\u7BA1\u7406");
tagCmd.command("list").alias("ls").description("\u5217\u51FA\u6807\u7B7E").option("-j, --json", "JSON\u683C\u5F0F").action((options) => {
  const tags = getDatabase().prepare("SELECT * FROM tags ORDER BY name").all();
  if (options.json) console.log(JSON.stringify(tags, null, 2));
  else tags.forEach((t2) => console.log(source_default.cyan(`  ${t2.name}`)));
});
tagCmd.command("add").description("\u6DFB\u52A0\u6807\u7B7E").argument("<name>", "\u6807\u7B7E\u540D").action((name) => {
  getDatabase().prepare("INSERT OR IGNORE INTO tags (name) VALUES (?)").run(name);
  console.log(source_default.green(`\u2713 \u6807\u7B7E\u5DF2\u6DFB\u52A0: ${name}`));
});
tagCmd.command("delete").alias("del").description("\u5220\u9664\u6807\u7B7E").argument("<name>", "\u6807\u7B7E\u540D").action((name) => {
  getDatabase().prepare("DELETE FROM tags WHERE name = ?").run(name);
  console.log(source_default.green(`\u2713 \u6807\u7B7E\u5DF2\u5220\u9664: ${name}`));
});
var subtaskCmd = program.command("subtask").description("\u5B50\u4EFB\u52A1\u7BA1\u7406");
subtaskCmd.command("list").alias("ls").description("\u5217\u51FA\u5B50\u4EFB\u52A1").argument("<todoId>", "\u4EFB\u52A1ID").option("-j, --json", "JSON\u683C\u5F0F").action((todoId, options) => {
  const subtasks = getDatabase().prepare("SELECT * FROM subtasks WHERE todoId = ? ORDER BY orderNum").all(todoId);
  if (options.json) console.log(JSON.stringify(subtasks.map((s) => ({ id: s.id, text: s.text, completed: s.completed === 1 })), null, 2));
  else subtasks.forEach((s) => console.log(`${s.completed === 1 ? "\u2713" : "\u25CB"} ${s.id} ${s.text}`));
});
subtaskCmd.command("add").description("\u6DFB\u52A0\u5B50\u4EFB\u52A1").argument("<todoId>", "\u4EFB\u52A1ID").argument("<text>", "\u6587\u672C").action((todoId, text) => {
  const id = Date.now().toString();
  const now = (/* @__PURE__ */ new Date()).toISOString();
  getDatabase().prepare("INSERT INTO subtasks (id, todoId, text, completed, orderNum, createdAt, updatedAt) VALUES (?, ?, ?, 0, 0, ?, ?)").run(id, todoId, text, now, now);
  console.log(source_default.green(`\u2713 \u5B50\u4EFB\u52A1\u5DF2\u6DFB\u52A0: ${text}`));
});
subtaskCmd.command("complete").description("\u5B8C\u6210\u5B50\u4EFB\u52A1").argument("<id>", "\u5B50\u4EFB\u52A1ID").action((id) => {
  getDatabase().prepare("UPDATE subtasks SET completed = 1, updatedAt = ? WHERE id = ?").run((/* @__PURE__ */ new Date()).toISOString(), id);
  console.log(source_default.green("\u2713 \u5B50\u4EFB\u52A1\u5DF2\u5B8C\u6210"));
});
subtaskCmd.command("delete").alias("del").description("\u5220\u9664\u5B50\u4EFB\u52A1").argument("<id>", "\u5B50\u4EFB\u52A1ID").action((id) => {
  getDatabase().prepare("DELETE FROM subtasks WHERE id = ?").run(id);
  console.log(source_default.green("\u2713 \u5B50\u4EFB\u52A1\u5DF2\u5220\u9664"));
});
var projectCmd = program.command("project").description("\u9879\u76EE\u7BA1\u7406");
projectCmd.command("list").alias("ls").description("\u5217\u51FA\u9879\u76EE").option("-j, --json", "JSON\u683C\u5F0F").action((options) => {
  const projects = getDatabase().prepare(`SELECT * FROM projects WHERE status = 'active' ORDER BY createdAt DESC`).all();
  if (options.json) console.log(JSON.stringify(projects, null, 2));
  else projects.forEach((p2) => console.log(source_default.cyan(`${p2.id} ${p2.name}`)));
});
projectCmd.command("add").description("\u6DFB\u52A0\u9879\u76EE").argument("<name>", "\u9879\u76EE\u540D").option("-d, --description <desc>", "\u63CF\u8FF0").action((name, options) => {
  const id = Date.now().toString();
  const now = (/* @__PURE__ */ new Date()).toISOString();
  getDatabase().prepare(`INSERT INTO projects (id, name, description, status, createdAt, updatedAt) VALUES (?, ?, ?, 'active', ?, ?)`).run(id, name, options.description || "", now, now);
  console.log(source_default.green(`\u2713 \u9879\u76EE\u5DF2\u6DFB\u52A0: ${name}`));
});
projectCmd.command("show").description("\u9879\u76EE\u8BE6\u60C5").argument("<id>", "\u9879\u76EEID").action((id) => {
  const project = getDatabase().prepare("SELECT * FROM projects WHERE id = ?").get(id);
  if (!project) {
    console.log(source_default.red("\u9879\u76EE\u4E0D\u5B58\u5728"));
    return;
  }
  console.log(source_default.bold(`\u9879\u76EE: ${project.name}`));
  console.log(source_default.gray(`\u63CF\u8FF0: ${project.description || "\u65E0"}`));
  console.log(source_default.gray(`\u72B6\u6001: ${project.status}`));
});
serverCmd.command("update").description("\u66F4\u65B0\u670D\u52A1\u5668").argument("<id>", "\u670D\u52A1\u5668ID").option("-n, --name <name>", "\u540D\u79F0").option("-h, --host <host>", "\u4E3B\u673A").option("-p, --port <port>", "\u7AEF\u53E3").action((id, options) => {
  const now = (/* @__PURE__ */ new Date()).toISOString();
  getDatabase().prepare("UPDATE servers SET name = COALESCE(?, name), host = COALESCE(?, host), port = COALESCE(?, port), updatedAt = ? WHERE id = ?").run(options.name || null, options.host || null, options.port || null, now, id);
  console.log(source_default.green("\u2713 \u670D\u52A1\u5668\u5DF2\u66F4\u65B0"));
});
serverCmd.command("show").description("\u670D\u52A1\u5668\u8BE6\u60C5").argument("<id>", "\u670D\u52A1\u5668ID").action((id) => {
  const server = getDatabase().prepare("SELECT * FROM servers WHERE id = ?").get(id);
  if (!server) {
    console.log(source_default.red("\u670D\u52A1\u5668\u4E0D\u5B58\u5728"));
    return;
  }
  console.log(source_default.bold(`\u670D\u52A1\u5668: ${server.name}`));
  console.log(source_default.gray(`\u4E3B\u673A: ${server.host}:${server.port}`));
  console.log(source_default.gray(`\u7528\u6237: ${server.username}`));
  console.log(source_default.gray(`\u6807\u7B7E: ${server.tags || "\u65E0"}`));
});
cicdCmd.command("list").alias("ls").description("\u5217\u51FA\u914D\u7F6E").option("-j, --json", "JSON\u683C\u5F0F").action((options) => {
  const configs = getDatabase().prepare("SELECT * FROM cicd_configs ORDER BY createdAt DESC").all();
  if (options.json) console.log(JSON.stringify(configs, null, 2));
  else configs.forEach((c3) => console.log(`${c3.id} ${c3.projectId} ${c3.deployBranch} ${c3.deployPath}`));
});
cicdCmd.command("status").description("\u90E8\u7F72\u72B6\u6001").argument("<projectId>", "\u9879\u76EEID").action((projectId) => {
  const log = getDatabase().prepare("SELECT * FROM deploy_logs WHERE projectId = ? ORDER BY createdAt DESC LIMIT 1").get(projectId);
  if (!log) {
    console.log(source_default.gray("\u6682\u65E0\u90E8\u7F72\u8BB0\u5F55"));
    return;
  }
  console.log(source_default.bold(`\u6700\u8FD1\u90E8\u7F72: ${log.status}`));
  console.log(source_default.gray(`\u65F6\u95F4: ${log.createdAt}`));
  if (log.errorMessage) console.log(source_default.red(`\u9519\u8BEF: ${log.errorMessage}`));
});
var dbCmd = program.command("db").description("\u6570\u636E\u5E93\u7BA1\u7406");
var connCmd = dbCmd.command("connections").alias("conn").description("\u6570\u636E\u5E93\u8FDE\u63A5\u7BA1\u7406");
connCmd.command("list").alias("ls").description("\u5217\u51FA\u6570\u636E\u5E93\u8FDE\u63A5").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((options) => {
  try {
    const row = getDatabase().prepare("SELECT value FROM settings WHERE key = 'db_connections'").get();
    let connections = [];
    if (row && row.value) {
      try {
        connections = JSON.parse(row.value);
      } catch (e) {
        console.error(source_default.red("\u89E3\u6790\u6570\u636E\u5E93\u8FDE\u63A5\u914D\u7F6E\u5931\u8D25"));
        return;
      }
    }
    if (connections.length === 0) {
      console.log(source_default.gray("\u6682\u65E0\u6570\u636E\u5E93\u8FDE\u63A5\u914D\u7F6E"));
      return;
    }
    if (options.json) {
      console.log(JSON.stringify(connections, null, 2));
      return;
    }
    console.log(source_default.bold("\n\u6570\u636E\u5E93\u8FDE\u63A5\u5217\u8868:\n"));
    connections.forEach((conn) => {
      const typeIcon = conn.type === "sqlite" ? source_default.green("\u{1F4C1}") : conn.type === "mysql" ? source_default.blue("\u{1F42C}") : conn.type === "postgresql" ? source_default.magenta("\u{1F418}") : source_default.yellow("\u2753");
      const typeLabel = conn.type ? conn.type.toUpperCase() : "UNKNOWN";
      console.log(source_default.cyan(`  ${typeIcon} ${conn.name || conn.id || "\u672A\u547D\u540D"}`) + source_default.gray(` [${typeLabel}]`));
      if (conn.type === "sqlite") {
        console.log(source_default.gray(`    \u8DEF\u5F84: ${conn.path || conn.host || "N/A"}`));
      } else {
        console.log(source_default.gray(`    \u4E3B\u673A: ${conn.host || "N/A"}:${conn.port || "N/A"}`));
        if (conn.database) console.log(source_default.gray(`    \u6570\u636E\u5E93: ${conn.database}`));
        console.log(source_default.yellow(`    \u63D0\u793A: \u8FDC\u7A0B\u6570\u636E\u5E93\u8BF7\u5728 GUI \u4E2D\u64CD\u4F5C`));
      }
      if (conn.id) console.log(source_default.gray(`    ID: ${conn.id}`));
      console.log("");
    });
  } catch (error) {
    console.error(source_default.red(`\u83B7\u53D6\u6570\u636E\u5E93\u8FDE\u63A5\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
dbCmd.command("query").alias("q").description("\u6267\u884C SQL \u67E5\u8BE2 (\u4EC5\u652F\u6301 SQLite \u8FDE\u63A5)").argument("<connection-id>", "\u8FDE\u63A5ID (local \u8868\u793A\u672C\u5730\u6570\u636E\u5E93)").argument("<sql>", "SQL \u8BED\u53E5").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((connectionId, sql, options) => {
  try {
    if (connectionId !== "local") {
      const row = getDatabase().prepare("SELECT value FROM settings WHERE key = 'db_connections'").get();
      let connections = [];
      if (row && row.value) {
        try {
          connections = JSON.parse(row.value);
        } catch (e) {
          connections = [];
        }
      }
      const conn = connections.find((c3) => c3.id === connectionId);
      if (!conn) {
        console.error(source_default.red(`\u8FDE\u63A5 "${connectionId}" \u4E0D\u5B58\u5728`));
        console.log(source_default.yellow('\u63D0\u793A: \u4F7F\u7528 "local" \u67E5\u8BE2\u672C\u5730\u6570\u636E\u5E93, \u8FDC\u7A0B\u8FDE\u63A5\u8BF7\u4F7F\u7528 GUI'));
        return;
      }
      if (conn.type !== "sqlite") {
        console.error(source_default.red(`\u8FDC\u7A0B ${conn.type} \u8FDE\u63A5\u65E0\u6CD5\u901A\u8FC7 CLI \u76F4\u63A5\u67E5\u8BE2`));
        console.log(source_default.yellow("\u8BF7\u5728 GUI \u4E2D\u4F7F\u7528\u6570\u636E\u5E93\u7BA1\u7406\u529F\u80FD"));
        return;
      }
    }
    const results = getDatabase().prepare(sql).all();
    if (options.json) {
      console.log(JSON.stringify(results, null, 2));
      return;
    }
    if (results.length === 0) {
      console.log(source_default.green("\u2713 \u67E5\u8BE2\u6210\u529F\uFF0C\u65E0\u8FD4\u56DE\u7ED3\u679C"));
      return;
    }
    const columns = Object.keys(results[0]);
    const colWidths = {};
    columns.forEach((col) => {
      colWidths[col] = Math.max(col.length, ...results.map((r2) => String(r2[col] ?? "").length));
    });
    const header = columns.map((col) => source_default.bold(col.padEnd(colWidths[col]))).join("  ");
    console.log(header);
    console.log(columns.map((col) => "\u2500".repeat(colWidths[col])).join("  "));
    results.forEach((row) => {
      const line = columns.map((col) => String(row[col] ?? "").padEnd(colWidths[col])).join("  ");
      console.log(line);
    });
    console.log(source_default.gray(`
\u5171 ${results.length} \u6761\u7ED3\u679C`));
  } catch (error) {
    console.error(source_default.red(`SQL \u6267\u884C\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
dbCmd.command("tables").description("\u5217\u51FA\u6240\u6709\u8868").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((options) => {
  try {
    const tables = getDatabase().prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name").all();
    if (options.json) {
      console.log(JSON.stringify(tables, null, 2));
      return;
    }
    if (tables.length === 0) {
      console.log(source_default.gray("\u6682\u65E0\u8868"));
      return;
    }
    console.log(source_default.bold("\n\u6570\u636E\u5E93\u8868:\n"));
    tables.forEach((t2) => {
      const count = getDatabase().prepare(`SELECT COUNT(*) as cnt FROM "${t2.name}"`).get();
      console.log(source_default.cyan(`  ${t2.name}`) + source_default.gray(` (${count.cnt} \u6761\u8BB0\u5F55)`));
    });
  } catch (error) {
    console.error(source_default.red(`\u83B7\u53D6\u8868\u5217\u8868\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
dbCmd.command("export").description("\u5BFC\u51FA\u8868\u6570\u636E\u4E3A JSON \u6216 CSV").argument("<table>", "\u8868\u540D").option("-j, --json", "\u5BFC\u51FA\u4E3A JSON \u683C\u5F0F (\u9ED8\u8BA4)").option("-c, --csv", "\u5BFC\u51FA\u4E3A CSV \u683C\u5F0F").option("-o, --output <file>", "\u8F93\u51FA\u6587\u4EF6\u8DEF\u5F84").action((table, options) => {
  try {
    const exists2 = getDatabase().prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=?").get(table);
    if (!exists2) {
      console.error(source_default.red(`\u8868 "${table}" \u4E0D\u5B58\u5728`));
      process.exit(1);
    }
    const rows = getDatabase().prepare(`SELECT * FROM "${table}"`).all();
    if (options.csv) {
      if (rows.length === 0) {
        console.log(source_default.gray("\u8868\u4E3A\u7A7A"));
        return;
      }
      const columns = Object.keys(rows[0]);
      const csvHeader = columns.join(",");
      const csvRows = rows.map(
        (row) => columns.map((col) => {
          const val = row[col] ?? "";
          return typeof val === "string" && (val.includes(",") || val.includes('"') || val.includes("\n")) ? `"${val.replace(/"/g, '""')}"` : String(val);
        }).join(",")
      );
      const csvContent = [csvHeader, ...csvRows].join("\n");
      if (options.output) {
        fs.writeFileSync(options.output, csvContent, "utf-8");
        console.log(source_default.green(`\u2713 \u5DF2\u5BFC\u51FA\u5230 ${options.output} (${rows.length} \u6761\u8BB0\u5F55)`));
      } else {
        console.log(csvContent);
      }
    } else {
      const jsonContent = JSON.stringify(rows, null, 2);
      if (options.output) {
        fs.writeFileSync(options.output, jsonContent, "utf-8");
        console.log(source_default.green(`\u2713 \u5DF2\u5BFC\u51FA\u5230 ${options.output} (${rows.length} \u6761\u8BB0\u5F55)`));
      } else {
        console.log(jsonContent);
      }
    }
  } catch (error) {
    console.error(source_default.red(`\u5BFC\u51FA\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
var redisCmd = dbCmd.command("redis").description("Redis \u7F13\u5B58\u7BA1\u7406");
redisCmd.command("get").description("\u83B7\u53D6 Redis \u952E\u503C").argument("<key>", "Redis \u952E\u540D").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((key, options) => {
  try {
    const row = getDatabase().prepare("SELECT value FROM settings WHERE key = 'redis_cache'").get();
    if (!row || !row.value) {
      console.log(source_default.yellow("Redis \u7F13\u5B58\u6570\u636E\u672A\u540C\u6B65\u5230\u672C\u5730"));
      console.log(source_default.gray("\u63D0\u793A: \u8BF7\u5728 GUI \u4E2D\u8FDE\u63A5 Redis \u670D\u52A1\u5668\u67E5\u770B"));
      return;
    }
    let cache;
    try {
      cache = JSON.parse(row.value);
    } catch (e) {
      cache = {};
    }
    if (!(key in cache)) {
      console.log(source_default.yellow(`\u952E "${key}" \u4E0D\u5B58\u5728`));
      return;
    }
    if (options.json) {
      console.log(JSON.stringify({ key, value: cache[key] }, null, 2));
    } else {
      console.log(source_default.bold(`\u952E: ${key}`));
      console.log(source_default.cyan(`\u503C: ${cache[key]}`));
    }
  } catch (error) {
    console.error(source_default.red(`\u83B7\u53D6 Redis \u952E\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
redisCmd.command("keys").description("\u5217\u51FA Redis \u952E").argument("[pattern]", "\u5339\u914D\u6A21\u5F0F (\u9ED8\u8BA4 *)").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((pattern, options) => {
  try {
    const row = getDatabase().prepare("SELECT value FROM settings WHERE key = 'redis_cache'").get();
    if (!row || !row.value) {
      console.log(source_default.yellow("Redis \u7F13\u5B58\u6570\u636E\u672A\u540C\u6B65\u5230\u672C\u5730"));
      console.log(source_default.gray("\u63D0\u793A: \u8BF7\u5728 GUI \u4E2D\u8FDE\u63A5 Redis \u670D\u52A1\u5668\u67E5\u770B"));
      return;
    }
    let cache;
    try {
      cache = JSON.parse(row.value);
    } catch (e) {
      cache = {};
    }
    const keys = Object.keys(cache);
    const filtered = pattern && pattern !== "*" ? keys.filter((k2) => {
      const regex = new RegExp("^" + pattern.replace(/[.+?^${}()|[\]\\]/g, "\\$&").replace(/\*/g, ".*") + "$");
      return regex.test(k2);
    }) : keys;
    if (options.json) {
      const result = {};
      filtered.forEach((k2) => result[k2] = cache[k2]);
      console.log(JSON.stringify(result, null, 2));
      return;
    }
    if (filtered.length === 0) {
      console.log(source_default.yellow("\u672A\u627E\u5230\u5339\u914D\u7684\u952E"));
      return;
    }
    console.log(source_default.bold(`
Redis \u952E (\u5171 ${filtered.length} \u4E2A):
`));
    filtered.forEach((k2) => {
      const val = String(cache[k2]);
      const display = val.length > 60 ? val.substring(0, 60) + "..." : val;
      console.log(source_default.cyan(`  ${k2}`) + source_default.gray(` = ${display}`));
    });
  } catch (error) {
    console.error(source_default.red(`\u5217\u51FA Redis \u952E\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
redisCmd.command("set").description("\u8BBE\u7F6E Redis \u952E\u503C (\u6807\u8BB0\u5F85\u540C\u6B65)").argument("<key>", "Redis \u952E\u540D").argument("<value>", "\u952E\u503C").option("-e, --expiry <seconds>", "\u8FC7\u671F\u65F6\u95F4 (\u79D2)").action((key, value, options) => {
  try {
    console.log(source_default.yellow(`\u26A0  Redis CLI \u8BBE\u7F6E\u64CD\u4F5C\u4EC5\u6807\u8BB0\u5F85\u540C\u6B65, \u5B9E\u9645\u5199\u5165\u9700\u5728 GUI \u4E2D\u5B8C\u6210`));
    const entry = {
      key,
      value,
      pending: true,
      timestamp: (/* @__PURE__ */ new Date()).toISOString(),
      expiry: options.expiry ? parseInt(options.expiry) : null
    };
    const row = getDatabase().prepare("SELECT value FROM settings WHERE key = 'redis_pending'").get();
    let pending;
    try {
      pending = row && row.value ? JSON.parse(row.value) : [];
    } catch (e) {
      pending = [];
    }
    pending = pending.filter((p2) => p2.key !== key);
    pending.push(entry);
    getDatabase().prepare("INSERT OR REPLACE INTO settings (key, value) VALUES ('redis_pending', ?)").run(JSON.stringify(pending));
    console.log(source_default.green(`\u2713 \u952E "${key}" \u5DF2\u6807\u8BB0\u5F85\u540C\u6B65\u5230 Redis`));
  } catch (error) {
    console.error(source_default.red(`\u8BBE\u7F6E Redis \u952E\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
redisCmd.command("del").description("\u5220\u9664 Redis \u952E (\u6807\u8BB0\u5F85\u540C\u6B65)").argument("<key>", "Redis \u952E\u540D").action((key) => {
  try {
    console.log(source_default.yellow(`\u26A0  Redis CLI \u5220\u9664\u64CD\u4F5C\u4EC5\u6807\u8BB0\u5F85\u540C\u6B65, \u5B9E\u9645\u5220\u9664\u9700\u5728 GUI \u4E2D\u5B8C\u6210`));
    const row = getDatabase().prepare("SELECT value FROM settings WHERE key = 'redis_cache'").get();
    if (row && row.value) {
      let cache;
      try {
        cache = JSON.parse(row.value);
      } catch (e) {
        cache = {};
      }
      if (key in cache) {
        delete cache[key];
        getDatabase().prepare("INSERT OR REPLACE INTO settings (key, value) VALUES ('redis_cache', ?)").run(JSON.stringify(cache));
      }
    }
    const pRow = getDatabase().prepare("SELECT value FROM settings WHERE key = 'redis_pending'").get();
    let pending;
    try {
      pending = pRow && pRow.value ? JSON.parse(pRow.value) : [];
    } catch (e) {
      pending = [];
    }
    pending.push({
      key,
      action: "delete",
      pending: true,
      timestamp: (/* @__PURE__ */ new Date()).toISOString()
    });
    getDatabase().prepare("INSERT OR REPLACE INTO settings (key, value) VALUES ('redis_pending', ?)").run(JSON.stringify(pending));
    console.log(source_default.green(`\u2713 \u952E "${key}" \u5DF2\u6807\u8BB0\u5F85\u4ECE Redis \u5220\u9664`));
  } catch (error) {
    console.error(source_default.red(`\u5220\u9664 Redis \u952E\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
var weeklyCmd = program.command("weekly").description("\u5468\u62A5\u7BA1\u7406");
weeklyCmd.command("list").alias("ls").description("\u5217\u51FA\u5468\u62A5").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").option("-l, --limit <n>", "\u663E\u793A\u6570\u91CF", "10").action((options) => {
  try {
    const limit = parseInt(options.limit) || 10;
    const reports = getDatabase().prepare(
      "SELECT * FROM weekly_reports ORDER BY createdAt DESC LIMIT ?"
    ).all(limit);
    if (options.json) {
      console.log(JSON.stringify(reports, null, 2));
      return;
    }
    if (reports.length === 0) {
      console.log(source_default.yellow("\u6682\u65E0\u5468\u62A5\u8BB0\u5F55"));
      return;
    }
    console.log(source_default.bold.cyan("\n\u{1F4CB} \u5468\u62A5\u5217\u8868\n"));
    reports.forEach((r2) => {
      const data = JSON.parse(r2.data || "{}");
      const completedCount = data.completedCount || 0;
      const totalCount = data.totalCount || 0;
      console.log(source_default.bold(`  #${r2.id}`) + source_default.gray(`  ${r2.startDate} ~ ${r2.endDate}`));
      console.log(source_default.gray(`    \u5B8C\u6210: ${completedCount}/${totalCount}  \u521B\u5EFA: ${r2.createdAt}`));
      if (data.highlights && data.highlights.length > 0) {
        console.log(source_default.gray(`    \u4EAE\u70B9: ${data.highlights.slice(0, 2).join("; ")}`));
      }
      console.log("");
    });
  } catch (error) {
    console.error(source_default.red(`\u5217\u51FA\u5468\u62A5\u5931\u8D25: ${error.message}`));
  }
});
weeklyCmd.command("show").description("\u67E5\u770B\u5468\u62A5\u8BE6\u60C5").argument("<id>", "\u5468\u62A5ID").action((id) => {
  try {
    const report = getDatabase().prepare("SELECT * FROM weekly_reports WHERE id = ?").get(id);
    if (!report) {
      console.log(source_default.red("\u5468\u62A5\u4E0D\u5B58\u5728"));
      return;
    }
    const data = JSON.parse(report.data || "{}");
    console.log(source_default.bold.cyan(`
\u{1F4CB} \u5468\u62A5 #${report.id}
`));
    console.log(source_default.bold(`  \u5468\u671F: ${report.startDate} ~ ${report.endDate}`));
    console.log(source_default.gray(`  \u521B\u5EFA\u65F6\u95F4: ${report.createdAt}
`));
    if (data.completedTodos && data.completedTodos.length > 0) {
      console.log(source_default.bold.green("  \u2705 \u5DF2\u5B8C\u6210\u4EFB\u52A1:"));
      data.completedTodos.forEach((t2) => console.log(source_default.green(`    \u2713 ${t2.text}`)));
      console.log("");
    }
    if (data.pendingTodos && data.pendingTodos.length > 0) {
      console.log(source_default.bold.yellow("  \u23F3 \u8FDB\u884C\u4E2D\u4EFB\u52A1:"));
      data.pendingTodos.forEach((t2) => console.log(source_default.yellow(`    \u25CB ${t2.text}`)));
      console.log("");
    }
    if (data.highlights && data.highlights.length > 0) {
      console.log(source_default.bold.cyan("  \u2728 \u672C\u5468\u4EAE\u70B9:"));
      data.highlights.forEach((h2) => console.log(source_default.cyan(`    \u2605 ${h2}`)));
      console.log("");
    }
    if (data.summary) {
      console.log(source_default.bold(`  \u{1F4DD} \u603B\u7ED3:`));
      console.log(source_default.gray(`    ${data.summary}`));
    }
  } catch (error) {
    console.error(source_default.red(`\u67E5\u770B\u5468\u62A5\u5931\u8D25: ${error.message}`));
  }
});
weeklyCmd.command("export").description("\u5BFC\u51FA\u6307\u5B9A\u5468\u671F\u7684\u5468\u62A5\u6570\u636E\u4E3AJSON").argument("<startDate>", "\u5F00\u59CB\u65E5\u671F (YYYY-MM-DD)").argument("<endDate>", "\u7ED3\u675F\u65E5\u671F (YYYY-MM-DD)").option("-o, --output <file>", "\u8F93\u51FA\u6587\u4EF6\u8DEF\u5F84").action((startDate, endDate, options) => {
  try {
    const db2 = getDatabase();
    const todos = db2.prepare(
      `SELECT * FROM todos WHERE createdAt >= ? AND createdAt <= ? ORDER BY createdAt`
    ).all(startDate, endDate);
    const completedTodos = todos.filter((t2) => t2.completed === 1);
    const pendingTodos = todos.filter((t2) => t2.completed === 0);
    const exportData = {
      startDate,
      endDate,
      exportedAt: (/* @__PURE__ */ new Date()).toISOString(),
      summary: {
        total: todos.length,
        completed: completedTodos.length,
        pending: pendingTodos.length
      },
      completedTodos: completedTodos.map((t2) => ({ id: t2.id, text: t2.text, priority: t2.priority, tag: t2.tag })),
      pendingTodos: pendingTodos.map((t2) => ({ id: t2.id, text: t2.text, priority: t2.priority, tag: t2.tag }))
    };
    const outputFile = options.output || `weekly_${startDate}_${endDate}.json`;
    fs.writeFileSync(outputFile, JSON.stringify(exportData, null, 2));
    console.log(source_default.green(`\u2713 \u5468\u62A5\u6570\u636E\u5DF2\u5BFC\u51FA: ${outputFile}`));
    console.log(source_default.gray(`  \u603B\u8BA1: ${todos.length} | \u5B8C\u6210: ${completedTodos.length} | \u8FDB\u884C\u4E2D: ${pendingTodos.length}`));
  } catch (error) {
    console.error(source_default.red(`\u5BFC\u51FA\u5468\u62A5\u5931\u8D25: ${error.message}`));
  }
});
var backupCmd = program.command("backup").description("\u6570\u636E\u5907\u4EFD\u4E0E\u6062\u590D");
backupCmd.command("export").description("\u5BFC\u51FA\u6240\u6709\u6570\u636E\u5230JSON\u6587\u4EF6").option("-o, --output <file>", "\u8F93\u51FA\u6587\u4EF6\u8DEF\u5F84").action((options) => {
  try {
    const db2 = getDatabase();
    const timestamp = (/* @__PURE__ */ new Date()).toISOString().replace(/[:.]/g, "-").slice(0, 19);
    const outputFile = options.output || `todo-backup-${timestamp}.json`;
    const backupData = {
      exportedAt: (/* @__PURE__ */ new Date()).toISOString(),
      version: "1.0",
      tables: {}
    };
    const tables = ["todos", "tags", "settings", "users", "messages", "file_transfers", "servers", "cicd_configs", "deploy_logs", "subtasks", "projects", "weekly_reports"];
    for (const table of tables) {
      try {
        const rows = db2.prepare(`SELECT * FROM ${table}`).all();
        backupData.tables[table] = rows;
      } catch (e) {
        backupData.tables[table] = [];
      }
    }
    fs.writeFileSync(outputFile, JSON.stringify(backupData, null, 2));
    console.log(source_default.green(`\u2713 \u6570\u636E\u5DF2\u5BFC\u51FA: ${outputFile}`));
    const totalRows = Object.values(backupData.tables).reduce((sum, rows) => sum + rows.length, 0);
    console.log(source_default.gray(`  \u5171 ${tables.length} \u5F20\u8868, ${totalRows} \u6761\u8BB0\u5F55`));
  } catch (error) {
    console.error(source_default.red(`\u5BFC\u51FA\u5931\u8D25: ${error.message}`));
  }
});
backupCmd.command("import").description("\u4ECEJSON\u6587\u4EF6\u5BFC\u5165\u6570\u636E\uFF08\u5408\u5E76\u6A21\u5F0F\uFF09").argument("<file>", "\u5907\u4EFD\u6587\u4EF6\u8DEF\u5F84").option("--skip-existing", "\u8DF3\u8FC7\u5DF2\u5B58\u5728\u7684\u8BB0\u5F55").action((file, options) => {
  try {
    if (!fs.existsSync(file)) {
      console.log(source_default.red(`\u6587\u4EF6\u4E0D\u5B58\u5728: ${file}`));
      return;
    }
    const content = fs.readFileSync(file, "utf-8");
    const backupData = JSON.parse(content);
    if (!backupData.tables) {
      console.log(source_default.red("\u65E0\u6548\u7684\u5907\u4EFD\u6587\u4EF6\u683C\u5F0F"));
      return;
    }
    const db2 = getDatabase();
    let imported = 0;
    let skipped = 0;
    const skipExisting = options.skipExisting;
    for (const [table, rows] of Object.entries(backupData.tables)) {
      if (!rows || rows.length === 0) continue;
      try {
        db2.prepare(`SELECT COUNT(*) FROM ${table}`).get();
      } catch (e) {
        console.log(source_default.yellow(`  \u8DF3\u8FC7\u672A\u77E5\u8868: ${table}`));
        continue;
      }
      const columns = Object.keys(rows[0] || {});
      const placeholders = columns.map(() => "?").join(", ");
      const columnNames = columns.join(", ");
      const insertStmt = db2.prepare(`INSERT OR ${skipExisting ? "IGNORE" : "REPLACE"} INTO ${table} (${columnNames}) VALUES (${placeholders})`);
      const insertMany = db2.transaction((rowsBatch) => {
        for (const row of rowsBatch) {
          try {
            insertStmt.run(...columns.map((c3) => row[c3]));
            imported++;
          } catch (e) {
            if (e.code === "SQLITE_CONSTRAINT") {
              skipped++;
            } else {
              throw e;
            }
          }
        }
      });
      insertMany(rows);
    }
    console.log(source_default.green(`\u2713 \u6570\u636E\u5BFC\u5165\u5B8C\u6210`));
    console.log(source_default.gray(`  \u5BFC\u5165: ${imported} \u6761 | \u8DF3\u8FC7: ${skipped} \u6761`));
  } catch (error) {
    console.error(source_default.red(`\u5BFC\u5165\u5931\u8D25: ${error.message}`));
  }
});
backupCmd.command("export-csv").description("\u5BFC\u51FA\u5F85\u529E\u4E8B\u9879\u4E3ACSV\u6587\u4EF6").option("-o, --output <file>", "\u8F93\u51FA\u6587\u4EF6\u8DEF\u5F84").option("-a, --all", "\u5305\u542B\u5DF2\u5B8C\u6210\u548C\u672A\u5B8C\u6210 (\u9ED8\u8BA4\u4EC5\u672A\u5B8C\u6210)").action((options) => {
  try {
    const db2 = getDatabase();
    let query = "SELECT id, text, completed, priority, dueDate, description, tag, createdAt, updatedAt FROM todos";
    if (!options.all) {
      query += " WHERE completed = 0";
    }
    query += " ORDER BY orderNum ASC, createdAt DESC";
    const todos = db2.prepare(query).all();
    const timestamp = (/* @__PURE__ */ new Date()).toISOString().replace(/[:.]/g, "-").slice(0, 19);
    const outputFile = options.output || `todos-${timestamp}.csv`;
    const headers = ["ID", "\u5185\u5BB9", "\u72B6\u6001", "\u4F18\u5148\u7EA7", "\u622A\u6B62\u65E5\u671F", "\u63CF\u8FF0", "\u6807\u7B7E", "\u521B\u5EFA\u65F6\u95F4", "\u66F4\u65B0\u65F6\u95F4"];
    const escapeCsv = (val) => {
      const str = String(val ?? "");
      if (str.includes(",") || str.includes('"') || str.includes("\n")) {
        return '"' + str.replace(/"/g, '""') + '"';
      }
      return str;
    };
    const lines = [headers.join(",")];
    for (const t2 of todos) {
      lines.push([
        escapeCsv(t2.id),
        escapeCsv(t2.text),
        escapeCsv(t2.completed === 1 ? "\u5DF2\u5B8C\u6210" : "\u672A\u5B8C\u6210"),
        escapeCsv(t2.priority),
        escapeCsv(t2.dueDate || ""),
        escapeCsv(t2.description || ""),
        escapeCsv(t2.tag || ""),
        escapeCsv(t2.createdAt),
        escapeCsv(t2.updatedAt)
      ].join(","));
    }
    fs.writeFileSync(outputFile, "\uFEFF" + lines.join("\n"), "utf-8");
    console.log(source_default.green(`\u2713 CSV\u5DF2\u5BFC\u51FA: ${outputFile}`));
    console.log(source_default.gray(`  \u5171 ${todos.length} \u6761\u8BB0\u5F55`));
  } catch (error) {
    console.error(source_default.red(`\u5BFC\u51FACSV\u5931\u8D25: ${error.message}`));
  }
});
function buildSshConfig(server) {
  const config = {
    host: server.host,
    port: server.port || 22,
    username: server.username,
    readyTimeout: 3e4
  };
  if (server.sshKeyPath && fs.existsSync(server.sshKeyPath)) {
    config.privateKey = fs.readFileSync(server.sshKeyPath);
  } else if (server.password) {
    config.password = server.password;
  }
  return config;
}
serverCmd.command("exec").description("SSH\u6267\u884C\u8FDC\u7A0B\u547D\u4EE4").argument("<id>", "\u670D\u52A1\u5668ID").argument("<command>", "\u8981\u6267\u884C\u7684\u547D\u4EE4").action(async (id, command) => {
  try {
    const server = getDatabase().prepare("SELECT * FROM servers WHERE id = ?").get(id);
    if (!server) {
      console.log(source_default.red(`\u670D\u52A1\u5668\u4E0D\u5B58\u5728: ${id}`));
      return;
    }
    console.log(source_default.yellow(`\u6B63\u5728\u8FDE\u63A5\u5230 ${server.name} (${server.host}:${server.port})...`));
    const { Client } = await import("ssh2");
    const conn = new Client();
    conn.on("ready", () => {
      console.log(source_default.green("\u2713 SSH\u8FDE\u63A5\u6210\u529F"));
      console.log(source_default.gray(`\u6267\u884C: ${command}
`));
      console.log(source_default.bold("--- \u8F93\u51FA\u5F00\u59CB ---"));
      conn.exec(command, (err, stream) => {
        if (err) {
          console.log(source_default.red(`\u6267\u884C\u5931\u8D25: ${err.message}`));
          conn.end();
          return;
        }
        stream.on("close", (code) => {
          console.log(source_default.bold("--- \u8F93\u51FA\u7ED3\u675F ---"));
          console.log(source_default.gray(`\u9000\u51FA\u7801: ${code}`));
          conn.end();
        }).on("data", (data) => {
          process.stdout.write(data.toString());
        }).stderr.on("data", (data) => {
          process.stderr.write(source_default.red(data.toString()));
        });
      });
    }).on("error", (err) => {
      console.log(source_default.red(`SSH\u8FDE\u63A5\u5931\u8D25: ${err.message}`));
      process.exit(1);
    }).connect(buildSshConfig(server));
  } catch (error) {
    console.error(source_default.red(`\u6267\u884C\u547D\u4EE4\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
serverCmd.command("sftp-list").description("SFTP\u5217\u51FA\u8FDC\u7A0B\u76EE\u5F55").argument("<id>", "\u670D\u52A1\u5668ID").argument("<remotePath>", "\u8FDC\u7A0B\u8DEF\u5F84").action(async (id, remotePath) => {
  try {
    const server = getDatabase().prepare("SELECT * FROM servers WHERE id = ?").get(id);
    if (!server) {
      console.log(source_default.red(`\u670D\u52A1\u5668\u4E0D\u5B58\u5728: ${id}`));
      return;
    }
    console.log(source_default.yellow(`\u6B63\u5728\u8FDE\u63A5\u5230 ${server.name} (${server.host}:${server.port})...`));
    const { Client } = await import("ssh2");
    const conn = new Client();
    conn.on("ready", () => {
      conn.sftp((err, sftp) => {
        if (err) {
          console.log(source_default.red(`SFTP\u8FDE\u63A5\u5931\u8D25: ${err.message}`));
          conn.end();
          return;
        }
        sftp.readdir(remotePath, (err2, list) => {
          if (err2) {
            console.log(source_default.red(`\u8BFB\u53D6\u76EE\u5F55\u5931\u8D25: ${err2.message}`));
            conn.end();
            return;
          }
          console.log(source_default.bold(`
\u76EE\u5F55: ${remotePath}
`));
          if (list.length === 0) {
            console.log(source_default.gray("  (\u7A7A\u76EE\u5F55)"));
          } else {
            list.forEach((entry) => {
              const attrs = entry.attrs;
              const isDir = (attrs.mode & 16384) !== 0;
              const size = attrs.size;
              const name = entry.filename;
              const prefix = isDir ? source_default.cyan.bold("[D]") : source_default.gray("   ");
              let sizeStr;
              if (size >= 1073741824) sizeStr = `${(size / 1073741824).toFixed(1)}G`;
              else if (size >= 1048576) sizeStr = `${(size / 1048576).toFixed(1)}M`;
              else if (size >= 1024) sizeStr = `${(size / 1024).toFixed(1)}K`;
              else sizeStr = `${size}B`;
              console.log(`${prefix}  ${sizeStr.padEnd(10)}  ${name}`);
            });
          }
          console.log(source_default.gray(`
\u5171 ${list.length} \u9879`));
          conn.end();
        });
      });
    }).on("error", (err) => {
      console.log(source_default.red(`SSH\u8FDE\u63A5\u5931\u8D25: ${err.message}`));
      process.exit(1);
    }).connect(buildSshConfig(server));
  } catch (error) {
    console.error(source_default.red(`SFTP\u5217\u8868\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
serverCmd.command("sftp-download").description("SFTP\u4E0B\u8F7D\u8FDC\u7A0B\u6587\u4EF6\u5230\u672C\u5730").argument("<id>", "\u670D\u52A1\u5668ID").argument("<remotePath>", "\u8FDC\u7A0B\u6587\u4EF6\u8DEF\u5F84").argument("<localPath>", "\u672C\u5730\u4FDD\u5B58\u8DEF\u5F84").action(async (id, remotePath, localPath) => {
  try {
    const server = getDatabase().prepare("SELECT * FROM servers WHERE id = ?").get(id);
    if (!server) {
      console.log(source_default.red(`\u670D\u52A1\u5668\u4E0D\u5B58\u5728: ${id}`));
      return;
    }
    const localDir = path.dirname(localPath);
    if (!fs.existsSync(localDir)) {
      fs.mkdirSync(localDir, { recursive: true });
    }
    console.log(source_default.yellow(`\u6B63\u5728\u8FDE\u63A5\u5230 ${server.name} (${server.host}:${server.port})...`));
    const { Client } = await import("ssh2");
    const conn = new Client();
    conn.on("ready", () => {
      conn.sftp((err, sftp) => {
        if (err) {
          console.log(source_default.red(`SFTP\u8FDE\u63A5\u5931\u8D25: ${err.message}`));
          conn.end();
          return;
        }
        sftp.stat(remotePath, (err2, stats) => {
          if (err2) {
            console.log(source_default.red(`\u8FDC\u7A0B\u6587\u4EF6\u4E0D\u5B58\u5728: ${remotePath}`));
            conn.end();
            return;
          }
          const totalSize = stats.size;
          let downloaded = 0;
          const readStream = sftp.createReadStream(remotePath);
          const writeStream = fs.createWriteStream(localPath);
          readStream.on("data", (chunk) => {
            downloaded += chunk.length;
            const pct = (downloaded / totalSize * 100).toFixed(1);
            process.stdout.write(source_default.yellow(`\r\u4E0B\u8F7D\u8FDB\u5EA6: ${pct}% (${downloaded}/${totalSize} bytes)`));
          });
          readStream.on("end", () => {
            writeStream.end();
            console.log(source_default.green(`
\u2713 \u4E0B\u8F7D\u5B8C\u6210: ${localPath}`));
            conn.end();
          });
          writeStream.on("error", (err3) => {
            console.log(source_default.red(`
\u5199\u5165\u672C\u5730\u6587\u4EF6\u5931\u8D25: ${err3.message}`));
            conn.end();
          });
          readStream.on("error", (err3) => {
            console.log(source_default.red(`
\u4E0B\u8F7D\u5931\u8D25: ${err3.message}`));
            conn.end();
          });
          readStream.pipe(writeStream);
        });
      });
    }).on("error", (err) => {
      console.log(source_default.red(`SSH\u8FDE\u63A5\u5931\u8D25: ${err.message}`));
      process.exit(1);
    }).connect(buildSshConfig(server));
  } catch (error) {
    console.error(source_default.red(`SFTP\u4E0B\u8F7D\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
cicdCmd.command("rollback").description("\u56DE\u6EDA\u6307\u5B9A\u90E8\u7F72").argument("<projectId>", "\u9879\u76EEID").argument("<deployLogId>", "\u90E8\u7F72\u65E5\u5FD7ID").action((projectId, deployLogId) => {
  try {
    const db2 = getDatabase();
    const deployLog = db2.prepare("SELECT * FROM deploy_logs WHERE id = ? AND projectId = ?").get(deployLogId, projectId);
    if (!deployLog) {
      console.log(source_default.red(`\u90E8\u7F72\u65E5\u5FD7\u4E0D\u5B58\u5728: ${deployLogId}`));
      return;
    }
    if (deployLog.status === "rolled_back") {
      console.log(source_default.yellow("\u8BE5\u90E8\u7F72\u5DF2\u56DE\u6EDA"));
      return;
    }
    if (deployLog.status === "failed" || deployLog.status === "cancelled") {
      console.log(source_default.red("\u65E0\u6CD5\u56DE\u6EDA\u5931\u8D25\u6216\u5DF2\u53D6\u6D88\u7684\u90E8\u7F72"));
      return;
    }
    const now = (/* @__PURE__ */ new Date()).toISOString();
    db2.prepare("UPDATE deploy_logs SET status = ?, endTime = ? WHERE id = ?").run("rolled_back", now, deployLogId);
    const historyResult = db2.prepare(`
        UPDATE deploy_history SET rolledBack = 1, rolledBackAt = ?, status = 'rolled_back'
        WHERE configId = ? AND rolledBack = 0
        ORDER BY deployedAt DESC
      `).run(now, deployLog.configId);
    console.log(source_default.green(`\u2713 \u90E8\u7F72\u5DF2\u56DE\u6EDA`));
    console.log(source_default.gray(`  \u90E8\u7F72\u65E5\u5FD7: ${deployLogId}`));
    console.log(source_default.gray(`  \u9879\u76EE: ${projectId}`));
    console.log(source_default.gray(`  \u539F\u59CB\u72B6\u6001: ${deployLog.status}`));
    console.log(source_default.gray(`  \u56DE\u6EDA\u65F6\u95F4: ${now}`));
    if (historyResult.changes > 0) {
      console.log(source_default.gray(`  \u66F4\u65B0\u90E8\u7F72\u5386\u53F2: ${historyResult.changes} \u6761`));
    }
  } catch (error) {
    console.error(source_default.red(`\u56DE\u6EDA\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
cicdCmd.command("cancel").description("\u53D6\u6D88\u8FDB\u884C\u4E2D\u7684\u90E8\u7F72").argument("<projectId>", "\u9879\u76EEID").action((projectId) => {
  try {
    const db2 = getDatabase();
    const activeLog = db2.prepare(
      "SELECT * FROM deploy_logs WHERE projectId = ? AND status IN ('pending', 'running') ORDER BY createdAt DESC LIMIT 1"
    ).get(projectId);
    if (!activeLog) {
      console.log(source_default.yellow("\u6CA1\u6709\u8FDB\u884C\u4E2D\u7684\u90E8\u7F72"));
      return;
    }
    const now = (/* @__PURE__ */ new Date()).toISOString();
    db2.prepare("UPDATE deploy_logs SET status = ?, endTime = ? WHERE id = ?").run("cancelled", now, activeLog.id);
    console.log(source_default.green(`\u2713 \u90E8\u7F72\u5DF2\u53D6\u6D88`));
    console.log(source_default.gray(`  \u90E8\u7F72\u65E5\u5FD7: ${activeLog.id}`));
    console.log(source_default.gray(`  \u539F\u59CB\u72B6\u6001: ${activeLog.status}`));
    console.log(source_default.gray(`  \u53D6\u6D88\u65F6\u95F4: ${now}`));
  } catch (error) {
    console.error(source_default.red(`\u53D6\u6D88\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
var modulesCmd = cicdCmd.command("modules").description("\u90E8\u7F72\u6A21\u5757\u7BA1\u7406");
modulesCmd.command("list").alias("ls").description("\u5217\u51FA\u90E8\u7F72\u6A21\u5757").argument("<configId>", "CI/CD\u914D\u7F6EID").option("-j, --json", "JSON\u683C\u5F0F").action((configId, options) => {
  try {
    const db2 = getDatabase();
    const config = db2.prepare("SELECT * FROM cicd_configs WHERE id = ?").get(configId);
    if (!config) {
      console.log(source_default.red(`\u914D\u7F6E\u4E0D\u5B58\u5728: ${configId}`));
      return;
    }
    const modules = db2.prepare("SELECT * FROM deploy_modules WHERE configId = ? ORDER BY createdAt DESC").all(configId);
    if (options.json) {
      console.log(JSON.stringify(modules, null, 2));
      return;
    }
    if (modules.length === 0) {
      console.log(source_default.gray("\u6682\u65E0\u6A21\u5757"));
      return;
    }
    console.log(source_default.bold(`
\u90E8\u7F72\u6A21\u5757 (\u914D\u7F6E: ${configId}):
`));
    modules.forEach((m, i2) => {
      console.log(`${source_default.yellow(`  ${i2 + 1}.`)}` + source_default.cyan(` ${m.name}`));
      console.log(source_default.gray(`     ID: ${m.id}`));
      console.log(source_default.gray(`     \u8DEF\u5F84: ${m.modulePath}`));
      console.log(source_default.gray(`     \u521B\u5EFA: ${m.createdAt}`));
      console.log("");
    });
  } catch (error) {
    console.error(source_default.red(`\u5217\u51FA\u6A21\u5757\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
modulesCmd.command("add").description("\u6DFB\u52A0\u90E8\u7F72\u6A21\u5757").argument("<configId>", "CI/CD\u914D\u7F6EID").argument("<moduleName>", "\u6A21\u5757\u540D\u79F0").argument("<modulePath>", "\u6A21\u5757\u8DEF\u5F84").action((configId, moduleName, modulePath) => {
  try {
    const db2 = getDatabase();
    const config = db2.prepare("SELECT * FROM cicd_configs WHERE id = ?").get(configId);
    if (!config) {
      console.log(source_default.red(`\u914D\u7F6E\u4E0D\u5B58\u5728: ${configId}`));
      return;
    }
    const id = Date.now().toString();
    const now = (/* @__PURE__ */ new Date()).toISOString();
    db2.prepare("INSERT INTO deploy_modules (id, configId, name, modulePath, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?)").run(id, configId, moduleName, modulePath, now, now);
    console.log(source_default.green(`\u2713 \u6A21\u5757\u5DF2\u6DFB\u52A0: ${moduleName}`));
    console.log(source_default.gray(`  \u6A21\u5757ID: ${id}`));
    console.log(source_default.gray(`  \u914D\u7F6E: ${configId}`));
    console.log(source_default.gray(`  \u8DEF\u5F84: ${modulePath}`));
  } catch (error) {
    console.error(source_default.red(`\u6DFB\u52A0\u6A21\u5757\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
modulesCmd.command("delete").alias("del").description("\u5220\u9664\u90E8\u7F72\u6A21\u5757").argument("<moduleId>", "\u6A21\u5757ID").action((moduleId) => {
  try {
    const db2 = getDatabase();
    const module = db2.prepare("SELECT * FROM deploy_modules WHERE id = ?").get(moduleId);
    if (!module) {
      console.log(source_default.red(`\u6A21\u5757\u4E0D\u5B58\u5728: ${moduleId}`));
      return;
    }
    db2.prepare("DELETE FROM deploy_modules WHERE id = ?").run(moduleId);
    console.log(source_default.green(`\u2713 \u6A21\u5757\u5DF2\u5220\u9664: ${module.name} (${moduleId})`));
  } catch (error) {
    console.error(source_default.red(`\u5220\u9664\u6A21\u5757\u5931\u8D25: ${error.message}`));
    process.exit(1);
  }
});
projectCmd.command("update").description("\u66F4\u65B0\u9879\u76EE").argument("<id>", "\u9879\u76EEID").option("-n, --name <name>", "\u9879\u76EE\u540D").option("-d, --description <desc>", "\u63CF\u8FF0").option("-s, --status <status>", "\u72B6\u6001 (active/archived)").option("-g, --git-url <url>", "Git\u4ED3\u5E93\u5730\u5740").action((id, options) => {
  const db2 = getDatabase();
  const project = db2.prepare("SELECT * FROM projects WHERE id = ?").get(id);
  if (!project) {
    console.log(source_default.red("\u9879\u76EE\u4E0D\u5B58\u5728"));
    return;
  }
  const now = (/* @__PURE__ */ new Date()).toISOString();
  const fields = [];
  const values = [];
  if (options.name !== void 0) {
    fields.push("name = ?");
    values.push(options.name);
  }
  if (options.description !== void 0) {
    fields.push("description = ?");
    values.push(options.description);
  }
  if (options.status !== void 0) {
    if (!["active", "archived"].includes(options.status)) {
      console.log(source_default.red("\u65E0\u6548\u7684\u72B6\u6001\u503C\uFF0C\u5FC5\u987B\u4E3A active \u6216 archived"));
      return;
    }
    fields.push("status = ?");
    values.push(options.status);
  }
  if (options.gitUrl !== void 0) {
    fields.push("gitUrl = ?");
    values.push(options.gitUrl);
  }
  if (fields.length === 0) {
    console.log(source_default.yellow("\u6CA1\u6709\u63D0\u4F9B\u4EFB\u4F55\u66F4\u65B0\u9009\u9879"));
    return;
  }
  fields.push("updatedAt = ?");
  values.push(now);
  values.push(id);
  db2.prepare(`UPDATE projects SET ${fields.join(", ")} WHERE id = ?`).run(...values);
  console.log(source_default.green("\u2713 \u9879\u76EE\u5DF2\u66F4\u65B0"));
});
projectCmd.command("delete").alias("del").description("\u5220\u9664\u9879\u76EE (\u8F6F\u5220\u9664\uFF0C\u8BBE\u4E3A archived)").argument("<id>", "\u9879\u76EEID").action((id) => {
  const db2 = getDatabase();
  const project = db2.prepare("SELECT * FROM projects WHERE id = ?").get(id);
  if (!project) {
    console.log(source_default.red("\u9879\u76EE\u4E0D\u5B58\u5728"));
    return;
  }
  if (project.status === "archived") {
    console.log(source_default.yellow("\u8BE5\u9879\u76EE\u5DF2\u7ECF\u662F archived \u72B6\u6001"));
    return;
  }
  db2.prepare("UPDATE projects SET status = ?, updatedAt = ? WHERE id = ?").run("archived", (/* @__PURE__ */ new Date()).toISOString(), id);
  console.log(source_default.green(`\u2713 \u9879\u76EE\u5DF2\u5220\u9664 (\u8F6F\u5220\u9664): ${project.name}`));
});
subtaskCmd.command("update").description("\u66F4\u65B0\u5B50\u4EFB\u52A1").argument("<id>", "\u5B50\u4EFB\u52A1ID").option("-t, --text <text>", "\u5B50\u4EFB\u52A1\u6587\u672C").option("-c, --completed", "\u6807\u8BB0\u4E3A\u5DF2\u5B8C\u6210").action((id, options) => {
  const db2 = getDatabase();
  const subtask = db2.prepare("SELECT * FROM subtasks WHERE id = ?").get(id);
  if (!subtask) {
    console.log(source_default.red("\u5B50\u4EFB\u52A1\u4E0D\u5B58\u5728"));
    return;
  }
  const now = (/* @__PURE__ */ new Date()).toISOString();
  const fields = [];
  const values = [];
  if (options.text !== void 0) {
    fields.push("text = ?");
    values.push(options.text);
  }
  if (options.completed) {
    fields.push("completed = ?");
    values.push(1);
  }
  if (fields.length === 0) {
    console.log(source_default.yellow("\u6CA1\u6709\u63D0\u4F9B\u4EFB\u4F55\u66F4\u65B0\u9009\u9879"));
    return;
  }
  fields.push("updatedAt = ?");
  values.push(now);
  values.push(id);
  db2.prepare(`UPDATE subtasks SET ${fields.join(", ")} WHERE id = ?`).run(...values);
  console.log(source_default.green("\u2713 \u5B50\u4EFB\u52A1\u5DF2\u66F4\u65B0"));
});
subtaskCmd.command("toggle").description("\u5207\u6362\u5B50\u4EFB\u52A1\u5B8C\u6210\u72B6\u6001").argument("<id>", "\u5B50\u4EFB\u52A1ID").action((id) => {
  const db2 = getDatabase();
  const subtask = db2.prepare("SELECT * FROM subtasks WHERE id = ?").get(id);
  if (!subtask) {
    console.log(source_default.red("\u5B50\u4EFB\u52A1\u4E0D\u5B58\u5728"));
    return;
  }
  const now = (/* @__PURE__ */ new Date()).toISOString();
  const newCompleted = subtask.completed === 1 ? 0 : 1;
  db2.prepare("UPDATE subtasks SET completed = ?, updatedAt = ? WHERE id = ?").run(newCompleted, now, id);
  console.log(source_default.green(`\u2713 \u5B50\u4EFB\u52A1\u5DF2${newCompleted === 1 ? "\u5B8C\u6210" : "\u53D6\u6D88\u5B8C\u6210"}: ${subtask.text}`));
});
function printFullHelp() {
  console.log(source_default.bold("\n\u{1F4CB} SuperTool CLI \u2014 \u5B8C\u6574\u5E2E\u52A9\n"));
  console.log(source_default.underline(source_default.bold("\u57FA\u672C\u4EFB\u52A1\u7BA1\u7406")));
  console.log(source_default.yellow("  todo add <text>"));
  console.log("    \u6DFB\u52A0\u65B0\u4EFB\u52A1  [-p priority] [-d due-date] [-t tag]");
  console.log(source_default.yellow("  todo list|ls"));
  console.log("    \u5217\u51FA\u4EFB\u52A1  [-a all] [-c completed] [-t tag] [-p priority] [-j json]");
  console.log(source_default.yellow("  todo complete|done <id>"));
  console.log("    \u6807\u8BB0\u4EFB\u52A1\u5B8C\u6210");
  console.log(source_default.yellow("  todo delete|del <id>"));
  console.log("    \u5220\u9664\u4EFB\u52A1");
  console.log(source_default.yellow("  todo show|view <id>"));
  console.log("    \u663E\u793A\u4EFB\u52A1\u8BE6\u60C5");
  console.log(source_default.yellow("  todo edit|e <id>"));
  console.log("    \u7F16\u8F91\u4EFB\u52A1  [-t text] [-p priority] [-T tag]");
  console.log(source_default.yellow("  todo uncomplete|undo <id>"));
  console.log("    \u53D6\u6D88\u5B8C\u6210\u72B6\u6001");
  console.log(source_default.yellow("  todo stats|stat"));
  console.log("    \u663E\u793A\u4EFB\u52A1\u7EDF\u8BA1");
  console.log(source_default.yellow("  todo clear|clean"));
  console.log("    \u6E05\u7A7A\u5DF2\u5B8C\u6210\u4EFB\u52A1");
  console.log(source_default.yellow("  todo search|find <keyword>"));
  console.log("    \u641C\u7D22\u4EFB\u52A1");
  console.log("");
  console.log(source_default.underline(source_default.bold("\u5C40\u57DF\u7F51\u534F\u4F5C")));
  console.log(source_default.yellow("  todo peers"));
  console.log("    \u67E5\u770B\u5C40\u57DF\u7F51\u53D1\u73B0\u7684\u7528\u6237  [-j json]");
  console.log(source_default.yellow("  todo send-message <message>"));
  console.log("    \u53D1\u9001\u6D88\u606F  [-t to-user] [-j json]");
  console.log(source_default.yellow("  todo send-file <file-path> <user-id>"));
  console.log("    \u53D1\u9001\u6587\u4EF6  [-j json]");
  console.log(source_default.yellow("  todo messages|msgs"));
  console.log("    \u67E5\u770B\u6D88\u606F\u5386\u53F2  [-l limit] [-j json]");
  console.log("");
  console.log(source_default.underline(source_default.bold("\u6807\u7B7E\u7BA1\u7406")));
  console.log(source_default.yellow("  todo tag list|ls"));
  console.log("    \u5217\u51FA\u6240\u6709\u6807\u7B7E  [-j json]");
  console.log(source_default.yellow("  todo tag add <name>"));
  console.log("    \u6DFB\u52A0\u6807\u7B7E");
  console.log(source_default.yellow("  todo tag delete|del <name>"));
  console.log("    \u5220\u9664\u6807\u7B7E");
  console.log("");
  console.log(source_default.underline(source_default.bold("\u5B50\u4EFB\u52A1\u7BA1\u7406")));
  console.log(source_default.yellow("  todo subtask list|ls <todoId>"));
  console.log("    \u5217\u51FA\u5B50\u4EFB\u52A1  [-j json]");
  console.log(source_default.yellow("  todo subtask add <todoId> <text>"));
  console.log("    \u6DFB\u52A0\u5B50\u4EFB\u52A1");
  console.log(source_default.yellow("  todo subtask complete <id>"));
  console.log("    \u5B8C\u6210\u5B50\u4EFB\u52A1");
  console.log(source_default.yellow("  todo subtask update <id>"));
  console.log("    \u66F4\u65B0\u5B50\u4EFB\u52A1  [-t text] [-c completed]");
  console.log(source_default.yellow("  todo subtask toggle <id>"));
  console.log("    \u5207\u6362\u5B50\u4EFB\u52A1\u5B8C\u6210\u72B6\u6001");
  console.log(source_default.yellow("  todo subtask delete|del <id>"));
  console.log("    \u5220\u9664\u5B50\u4EFB\u52A1");
  console.log("");
  console.log(source_default.underline(source_default.bold("\u9879\u76EE\u7BA1\u7406")));
  console.log(source_default.yellow("  todo project list|ls"));
  console.log("    \u5217\u51FA\u6D3B\u8DC3\u9879\u76EE  [-j json]");
  console.log(source_default.yellow("  todo project add <name>"));
  console.log("    \u6DFB\u52A0\u9879\u76EE  [-d description]");
  console.log(source_default.yellow("  todo project show <id>"));
  console.log("    \u9879\u76EE\u8BE6\u60C5");
  console.log(source_default.yellow("  todo project update <id>"));
  console.log("    \u66F4\u65B0\u9879\u76EE  [-n name] [-d description] [-s status] [-g git-url]");
  console.log(source_default.yellow("  todo project delete|del <id>"));
  console.log("    \u5220\u9664\u9879\u76EE (\u8F6F\u5220\u9664)");
  console.log("");
  console.log(source_default.underline(source_default.bold("\u670D\u52A1\u5668\u7BA1\u7406")));
  console.log(source_default.yellow("  todo server list|ls"));
  console.log("    \u67E5\u770B\u6240\u6709\u670D\u52A1\u5668  [-j json]");
  console.log(source_default.yellow("  todo server add <name> <host>"));
  console.log("    \u6DFB\u52A0\u670D\u52A1\u5668  [-p port] [-u user] [-k key] [-w password] [-t tags]");
  console.log(source_default.yellow("  todo server delete|del <id>"));
  console.log("    \u5220\u9664\u670D\u52A1\u5668");
  console.log(source_default.yellow("  todo server test <id>"));
  console.log("    \u6D4B\u8BD5\u670D\u52A1\u5668\u8FDE\u63A5");
  console.log(source_default.yellow("  todo server update <id>"));
  console.log("    \u66F4\u65B0\u670D\u52A1\u5668  [-n name] [-h host] [-p port]");
  console.log(source_default.yellow("  todo server show <id>"));
  console.log("    \u670D\u52A1\u5668\u8BE6\u60C5");
  console.log(source_default.yellow("  todo server exec <id> <command>"));
  console.log("    \u6267\u884C\u8FDC\u7A0B\u547D\u4EE4");
  console.log(source_default.yellow("  todo server sftp-list <id> <remotePath>"));
  console.log("    \u5217\u51FA\u8FDC\u7A0B\u76EE\u5F55");
  console.log(source_default.yellow("  todo server sftp-download <id> <remote> <local>"));
  console.log("    \u4E0B\u8F7D\u8FDC\u7A0B\u6587\u4EF6");
  console.log("");
  console.log(source_default.underline(source_default.bold("CI/CD")));
  console.log(source_default.yellow("  todo cicd config <projectId>"));
  console.log("    \u914D\u7F6ECI/CD  [-b branch] [-p path] [-s server]");
  console.log(source_default.yellow("  todo cicd deploy <projectId>"));
  console.log("    \u6267\u884C\u90E8\u7F72  [-d dry-run]");
  console.log(source_default.yellow("  todo cicd logs <projectId>"));
  console.log("    \u67E5\u770B\u90E8\u7F72\u65E5\u5FD7  [-l limit]");
  console.log(source_default.yellow("  todo cicd list|ls"));
  console.log("    \u5217\u51FA\u914D\u7F6E  [-j json]");
  console.log(source_default.yellow("  todo cicd status <projectId>"));
  console.log("    \u67E5\u770B\u90E8\u7F72\u72B6\u6001");
  console.log(source_default.yellow("  todo cicd rollback <projectId> <deployLogId>"));
  console.log("    \u56DE\u6EDA\u90E8\u7F72");
  console.log(source_default.yellow("  todo cicd cancel <projectId>"));
  console.log("    \u53D6\u6D88\u90E8\u7F72");
  console.log(source_default.yellow("  todo cicd modules list|ls <configId>"));
  console.log("    \u5217\u51FA\u90E8\u7F72\u6A21\u5757  [-j json]");
  console.log(source_default.yellow("  todo cicd modules add <configId> <name> <path>"));
  console.log("    \u6DFB\u52A0\u90E8\u7F72\u6A21\u5757");
  console.log(source_default.yellow("  todo cicd modules delete|del <moduleId>"));
  console.log("    \u5220\u9664\u90E8\u7F72\u6A21\u5757");
  console.log("");
  console.log(source_default.underline(source_default.bold("\u5468\u62A5\u7BA1\u7406")));
  console.log(source_default.yellow("  todo weekly list|ls"));
  console.log("    \u5217\u51FA\u5468\u62A5  [-j json] [-l limit]");
  console.log(source_default.yellow("  todo weekly show <id>"));
  console.log("    \u67E5\u770B\u5468\u62A5\u8BE6\u60C5");
  console.log(source_default.yellow("  todo weekly export <startDate> <endDate>"));
  console.log("    \u5BFC\u51FA\u5468\u62A5\u6570\u636E  [-o output]");
  console.log("");
  console.log(source_default.underline(source_default.bold("\u6570\u636E\u5E93\u7BA1\u7406")));
  console.log(source_default.yellow("  todo db connections|conn ls"));
  console.log("    \u5217\u51FA\u6570\u636E\u5E93\u8FDE\u63A5  [-j json]");
  console.log(source_default.yellow("  todo db query|q <conn-id> <sql>"));
  console.log("    \u6267\u884CSQL\u67E5\u8BE2 (\u4EC5SQLite)  [-j json]");
  console.log(source_default.yellow("  todo db tables"));
  console.log("    \u5217\u51FA\u6240\u6709\u8868  [-j json]");
  console.log(source_default.yellow("  todo db export <table>"));
  console.log("    \u5BFC\u51FA\u8868\u6570\u636E  [-j json] [-c csv] [-o output]");
  console.log(source_default.yellow("  todo db redis get <key>"));
  console.log("    \u83B7\u53D6Redis\u952E\u503C  [-j json]");
  console.log(source_default.yellow("  todo db redis keys [pattern]"));
  console.log("    \u5217\u51FARedis\u952E  [-j json]");
  console.log(source_default.yellow("  todo db redis set <key> <value>"));
  console.log("    \u8BBE\u7F6ERedis\u952E  [-e expiry]");
  console.log(source_default.yellow("  todo db redis del <key>"));
  console.log("    \u5220\u9664Redis\u952E");
  console.log("");
  console.log(source_default.underline(source_default.bold("\u6570\u636E\u5907\u4EFD")));
  console.log(source_default.yellow("  todo backup export"));
  console.log("    \u5BFC\u51FA\u6240\u6709\u6570\u636E\u5230JSON  [-o output]");
  console.log(source_default.yellow("  todo backup import <file>"));
  console.log("    \u4ECEJSON\u6587\u4EF6\u5BFC\u5165\u6570\u636E");
  console.log(source_default.yellow("  todo backup export-csv"));
  console.log("    \u5BFC\u51FA\u4EFB\u52A1\u4E3ACSV  [-a all]");
  console.log("");
  console.log(source_default.underline(source_default.bold("\u7B14\u8BB0\u7BA1\u7406")));
  console.log(source_default.yellow("  todo note list|ls"));
  console.log("    \u5217\u51FA\u7B14\u8BB0  [-q query] [-g group] [-j json]");
  console.log(source_default.yellow("  todo note add [title]"));
  console.log("    \u65B0\u5EFA\u7B14\u8BB0  [-c content] [-g group]");
  console.log(source_default.yellow("  todo note show <id>"));
  console.log("    \u67E5\u770B\u7B14\u8BB0\u8BE6\u60C5");
  console.log(source_default.yellow("  todo note edit <id>"));
  console.log("    \u7F16\u8F91\u7B14\u8BB0  [-t title] [-c content] [-g group] [--pin] [--unpin]");
  console.log(source_default.yellow("  todo note delete|del <id>"));
  console.log("    \u5220\u9664\u7B14\u8BB0");
  console.log(source_default.yellow("  todo note search <query>"));
  console.log("    \u641C\u7D22\u7B14\u8BB0");
  console.log(source_default.yellow("  todo note groups ls"));
  console.log("    \u5217\u51FA\u7B14\u8BB0\u5206\u7EC4");
  console.log(source_default.yellow("  todo note groups add <name>"));
  console.log("    \u65B0\u5EFA\u5206\u7EC4  [-i icon]");
  console.log(source_default.yellow("  todo note groups delete|del <id>"));
  console.log("    \u5220\u9664\u5206\u7EC4");
  console.log("");
  console.log(source_default.underline(source_default.bold("MFA/\u9A8C\u8BC1\u7801")));
  console.log(source_default.yellow("  todo mfa list|ls"));
  console.log("    \u5217\u51FAMFA\u5BC6\u94A5  [-j json]");
  console.log(source_default.yellow("  todo mfa add <name> <secret>"));
  console.log("    \u6DFB\u52A0MFA\u5BC6\u94A5  [-i issuer] [-a account] [-d digits] [-p period] [-u uri]");
  console.log(source_default.yellow("  todo mfa code <name>"));
  console.log("    \u751F\u6210\u5F53\u524D\u9A8C\u8BC1\u7801");
  console.log(source_default.yellow("  todo mfa delete|del <name>"));
  console.log("    \u5220\u9664MFA\u5BC6\u94A5");
  console.log("");
  console.log(source_default.underline(source_default.bold("Git\u540C\u6B65")));
  console.log(source_default.yellow("  todo git-sync status"));
  console.log("    \u67E5\u770B\u540C\u6B65\u72B6\u6001");
  console.log(source_default.yellow("  todo git-sync configure"));
  console.log("    \u914D\u7F6E\u540C\u6B65  [-r remote] [-b branch] [-i interval] [--enable] [--disable]");
  console.log(source_default.yellow("  todo git-sync sync|push"));
  console.log("    \u624B\u52A8\u63A8\u9001\u6570\u636E");
  console.log(source_default.yellow("  todo git-sync pull"));
  console.log("    \u4ECE\u8FDC\u7A0B\u62C9\u53D6\u6570\u636E");
  console.log("");
  console.log(source_default.underline(source_default.bold("\u670D\u52A1\u5668\u5206\u7EC4")));
  console.log(source_default.yellow("  todo server-group list|ls"));
  console.log("    \u5217\u51FA\u670D\u52A1\u5668\u5206\u7EC4");
  console.log(source_default.yellow("  todo server-group add <name>"));
  console.log("    \u65B0\u5EFA\u670D\u52A1\u5668\u5206\u7EC4  [-d description]");
  console.log(source_default.yellow("  todo server-group delete|del <id>"));
  console.log("    \u5220\u9664\u670D\u52A1\u5668\u5206\u7EC4");
  console.log("");
  console.log(source_default.underline(source_default.bold("\u90E8\u7F72\u5386\u53F2")));
  console.log(source_default.yellow("  todo deploy-history list|ls"));
  console.log("    \u5217\u51FA\u90E8\u7F72\u5386\u53F2  [-p project] [-l limit] [-j json]");
  console.log(source_default.yellow("  todo deploy-history rollback <id>"));
  console.log("    \u56DE\u6EDA\u5230\u6307\u5B9A\u7248\u672C");
  console.log("");
  console.log(source_default.underline(source_default.bold("\u901A\u77E5\u7BA1\u7406")));
  console.log(source_default.yellow("  todo notification list|ls"));
  console.log("    \u67E5\u770B\u901A\u77E5\u8BBE\u7F6E");
  console.log(source_default.yellow("  todo notification config"));
  console.log("    \u914D\u7F6E\u901A\u77E5  [-t time]");
  console.log("");
  console.log(source_default.underline(source_default.bold("\u901A\u7528")));
  console.log(source_default.yellow("  todo version|v"));
  console.log("    \u663E\u793A\u7248\u672C\u53F7");
  console.log(source_default.yellow("  todo help-detail|help"));
  console.log("    \u663E\u793A\u6B64\u5E2E\u52A9\u4FE1\u606F");
  console.log(source_default.yellow("  -h, --help"));
  console.log("    \u663E\u793A\u5E2E\u52A9");
  console.log(source_default.yellow("  -V, --version"));
  console.log("    \u663E\u793A\u7248\u672C\u53F7");
  console.log("");
}
function printNotes(notes, options = {}) {
  if (options.json) {
    console.log(JSON.stringify(notes, null, 2));
    return;
  }
  if (notes.length === 0) {
    console.log(source_default.gray("\u6682\u65E0\u7B14\u8BB0"));
    return;
  }
  console.log(source_default.underline("\u7B14\u8BB0\u5217\u8868"));
  notes.forEach((n, i2) => {
    const pin = n.pinned ? source_default.yellow("\u{1F4CC} ") : "";
    const preview = (n.content || "").replace(/[#*`>\\-\\[\\]()!]/g, "").slice(0, 60);
    console.log(`  ${pin}${source_default.bold(n.title || "\u65E0\u6807\u9898")} ${source_default.gray("| " + formatDate(n.updatedAt))}`);
    if (preview) console.log(`    ${source_default.gray(preview)}`);
    console.log(`    ${source_default.gray("ID: " + n.id)}`);
  });
}
var _totpModule = null;
function requireTotpModule() {
  if (_totpModule) return _totpModule;
  try {
    const totpPath = path.join(__dirname, "dist-electron", "services", "totp.js");
    if (fs.existsSync(totpPath)) {
      _totpModule = __require(totpPath);
    } else {
      _totpModule = {
        generateTOTP: (secret, opts = {}) => {
          const crypto = __require("crypto");
          const digits = opts.digits || 6;
          const period = opts.period || 30;
          const algorithm = (opts.algorithm || "sha1").toLowerCase();
          const epoch = Math.floor(Date.now() / 1e3);
          const time = Math.floor(epoch / period);
          const timeBuffer = Buffer.alloc(8);
          timeBuffer.writeBigUInt64BE(BigInt(time));
          const hash = crypto.createHmac(algorithm, base32Decode(secret)).update(timeBuffer).digest();
          const offset = hash[hash.length - 1] & 15;
          const binary = (hash[offset] & 127) << 24 | (hash[offset + 1] & 255) << 16 | (hash[offset + 2] & 255) << 8 | hash[offset + 3] & 255;
          return (binary % Math.pow(10, digits)).toString().padStart(digits, "0");
        },
        getRemainingTime: (period = 30) => period - Math.floor(Date.now() / 1e3) % period,
        parseOtpauthUri: (uri) => {
          const url = new URL(uri);
          const parts = decodeURIComponent(url.pathname.slice(1)).split(":");
          return {
            name: parts[0] || "Unknown",
            secret: url.searchParams.get("secret") || "",
            issuer: url.searchParams.get("issuer") || parts[0] || "",
            account: parts[1] || "",
            digits: parseInt(url.searchParams.get("digits") || "6"),
            period: parseInt(url.searchParams.get("period") || "30"),
            algorithm: url.searchParams.get("algorithm") || "sha1"
          };
        },
        validateBase32: (str) => /^[A-Z2-7]+=*$/i.test(str),
        formatCode: (code) => code.match(/.{1,3}/g)?.join(" ") || code
      };
    }
  } catch (e) {
    _totpModule = {
      generateTOTP: () => "000000",
      getRemainingTime: () => 0,
      parseOtpauthUri: () => ({ name: "", secret: "", issuer: "", account: "", digits: 6, period: 30, algorithm: "sha1" }),
      validateBase32: () => true,
      formatCode: (c3) => c3
    };
  }
  return _totpModule;
}
function base32Decode(base32) {
  const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
  let bits = "";
  const clean = base32.toUpperCase().replace(/=+$/, "");
  for (const char of clean) {
    const val = alphabet.indexOf(char);
    if (val === -1) continue;
    bits += val.toString(2).padStart(5, "0");
  }
  const bytes = [];
  for (let i2 = 0; i2 + 8 <= bits.length; i2 += 8) {
    bytes.push(parseInt(bits.slice(i2, i2 + 8), 2));
  }
  return Buffer.from(bytes);
}
function exportAllDataFromDb() {
  const db2 = getDatabase();
  const todos = db2.prepare("SELECT * FROM todos").all();
  const tags = db2.prepare("SELECT name FROM tags").all().map((t2) => t2.name);
  const subtasks = db2.prepare("SELECT * FROM subtasks").all();
  const settings = {};
  db2.prepare("SELECT * FROM settings").all().forEach((s) => {
    settings[s.key] = s.value;
  });
  const projects = db2.prepare("SELECT * FROM projects").all();
  const servers = db2.prepare("SELECT * FROM servers").all();
  const cicdConfigs = db2.prepare("SELECT * FROM cicd_configs").all();
  const deployModules = db2.prepare("SELECT * FROM deploy_modules").all();
  const deployLogs = db2.prepare("SELECT * FROM deploy_logs").all();
  const deployHistory = db2.prepare("SELECT * FROM deploy_history").all();
  const weeklyReports = db2.prepare("SELECT * FROM weekly_reports").all();
  let notes = [], noteGroups = [], mfaSecrets = [];
  try {
    notes = db2.prepare("SELECT * FROM notes").all();
  } catch {
  }
  try {
    noteGroups = db2.prepare("SELECT * FROM note_groups").all();
  } catch {
  }
  try {
    mfaSecrets = db2.prepare("SELECT * FROM mfa_secrets").all();
  } catch {
  }
  return {
    version: "2.0",
    exportedAt: (/* @__PURE__ */ new Date()).toISOString(),
    todos,
    subtasks,
    tags,
    settings,
    projects,
    servers,
    cicdConfigs,
    deployModules,
    deployLogs,
    deployHistory,
    weeklyReports,
    notes,
    noteGroups,
    mfaSecrets
  };
}
function importDataToDb(data, mode = "merge") {
  const db2 = getDatabase();
  let imported = 0, skipped = 0;
  if (mode === "replace") {
    db2.exec("DELETE FROM deploy_step_logs; DELETE FROM deploy_logs; DELETE FROM deploy_modules; DELETE FROM cicd_configs; DELETE FROM deploy_history; DELETE FROM weekly_reports; DELETE FROM subtasks; DELETE FROM notes; DELETE FROM note_groups; DELETE FROM mfa_secrets; DELETE FROM servers; DELETE FROM todos; DELETE FROM projects; DELETE FROM settings;");
  }
  if (data.projects) {
    for (const p2 of data.projects) {
      if (mode === "merge" && db2.prepare("SELECT id FROM projects WHERE id = ?").get(p2.id)) {
        skipped++;
        continue;
      }
      db2.prepare("INSERT OR REPLACE INTO projects (id, name, description, status, gitUrl, startDate, endDate, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)").run(p2.id, p2.name, p2.description || "", p2.status || "active", p2.gitUrl || null, p2.startDate || null, p2.endDate || null, p2.createdAt, p2.updatedAt);
      imported++;
    }
  }
  if (data.todos) {
    for (const t2 of data.todos) {
      if (mode === "merge" && db2.prepare("SELECT id FROM todos WHERE id = ?").get(t2.id)) {
        skipped++;
        continue;
      }
      db2.prepare("INSERT OR REPLACE INTO todos (id, text, completed, priority, dueDate, description, tag, createdAt, updatedAt, completedAt, assignedTo, assignedBy, assignedAt, owner, orderNum, repeatType, repeatInterval, repeatEndDate, repeatCount, parentTodoId, markdownDescription) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)").run(t2.id, t2.text, t2.completed ? 1 : 0, t2.priority, t2.dueDate || null, t2.description || "", t2.tag || "", t2.createdAt, t2.updatedAt, t2.completedAt || null, t2.assignedTo || "", t2.assignedBy || "", t2.assignedAt || null, t2.owner || "", t2.orderNum || 0, t2.repeatType || "", t2.repeatInterval || 1, t2.repeatEndDate || null, t2.repeatCount || -1, t2.parentTodoId || null, t2.markdownDescription || "");
      imported++;
    }
  }
  if (data.subtasks) {
    for (const s of data.subtasks) {
      if (mode === "merge" && db2.prepare("SELECT id FROM subtasks WHERE id = ?").get(s.id)) {
        skipped++;
        continue;
      }
      db2.prepare("INSERT OR REPLACE INTO subtasks (id, todoId, text, completed, orderNum, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?, ?)").run(s.id, s.todoId, s.text, s.completed ? 1 : 0, s.orderNum || 0, s.createdAt, s.updatedAt);
      imported++;
    }
  }
  if (data.tags) {
    for (const tag of data.tags) {
      db2.prepare("INSERT OR IGNORE INTO tags (name, createdAt) VALUES (?, ?)").run(tag, (/* @__PURE__ */ new Date()).toISOString());
      imported++;
    }
  }
  if (data.notes) {
    for (const n of data.notes) {
      if (mode === "merge" && db2.prepare("SELECT id FROM notes WHERE id = ?").get(n.id)) {
        skipped++;
        continue;
      }
      db2.prepare("INSERT OR REPLACE INTO notes (id, title, content, tags, pinned, groupId, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?)").run(n.id, n.title || "", n.content || "", n.tags || "[]", n.pinned || 0, n.groupId || null, n.createdAt, n.updatedAt);
      imported++;
    }
  }
  if (data.noteGroups) {
    for (const g of data.noteGroups) {
      if (mode === "merge" && db2.prepare("SELECT id FROM note_groups WHERE id = ?").get(g.id)) {
        skipped++;
        continue;
      }
      db2.prepare("INSERT OR REPLACE INTO note_groups (id, name, icon, sortOrder, createdAt) VALUES (?, ?, ?, ?, ?)").run(g.id, g.name, g.icon || "", g.sortOrder || 0, g.createdAt);
      imported++;
    }
  }
  if (data.mfaSecrets) {
    for (const m of data.mfaSecrets) {
      if (mode === "merge" && db2.prepare("SELECT id FROM mfa_secrets WHERE id = ?").get(m.id)) {
        skipped++;
        continue;
      }
      db2.prepare("INSERT OR REPLACE INTO mfa_secrets (id, name, secret, digits, period, algorithm, account, issuer, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)").run(m.id, m.name, m.secret, m.digits || 6, m.period || 30, m.algorithm || "sha1", m.account || "", m.issuer || "", m.createdAt, m.updatedAt);
      imported++;
    }
  }
  return { imported, skipped };
}
var notesCmd = program.command("note").alias("notes").description("\u7B14\u8BB0\u7BA1\u7406");
notesCmd.command("list").alias("ls").description("\u5217\u51FA\u7B14\u8BB0").option("-q, --query <text>", "\u641C\u7D22\u5173\u952E\u8BCD").option("-g, --group <id>", "\u6309\u5206\u7EC4\u8FC7\u6EE4").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((options) => {
  try {
    const db2 = getDatabase();
    let sql = "SELECT * FROM notes ORDER BY pinned DESC, updatedAt DESC";
    const params = [];
    if (options.query) {
      sql = "SELECT * FROM notes WHERE title LIKE ? OR content LIKE ? ORDER BY pinned DESC, updatedAt DESC";
      params.push(`%${options.query}%`, `%${options.query}%`);
    }
    const notes = db2.prepare(sql).all(...params);
    if (options.group) {
      const filtered = notes.filter((n) => n.groupId === options.group);
      printNotes(filtered, options);
    } else {
      printNotes(notes, options);
    }
  } catch (error) {
    console.error(source_default.red(`\u5217\u51FA\u7B14\u8BB0\u5931\u8D25: ${error.message}`));
  }
});
notesCmd.command("add").description("\u65B0\u5EFA\u7B14\u8BB0").argument("[title]", "\u7B14\u8BB0\u6807\u9898").option("-c, --content <text>", "\u7B14\u8BB0\u5185\u5BB9 (Markdown)").option("-g, --group <id>", "\u6240\u5C5E\u5206\u7EC4").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action(async (title, options) => {
  try {
    const id = `n_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
    const now = (/* @__PURE__ */ new Date()).toISOString();
    const titleText = title || "\u65E0\u6807\u9898";
    const content = options.content || "";
    const db2 = getDatabase();
    db2.prepare("INSERT INTO notes (id, title, content, tags, pinned, groupId, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?)").run(id, titleText, content, "[]", 0, options.group || null, now, now);
    if (options.json) {
      console.log(JSON.stringify({ id, title: titleText }, null, 2));
    } else {
      console.log(source_default.green(`\u2713 \u7B14\u8BB0\u5DF2\u521B\u5EFA: ${titleText} (${id})`));
    }
  } catch (error) {
    console.error(source_default.red(`\u521B\u5EFA\u7B14\u8BB0\u5931\u8D25: ${error.message}`));
  }
});
notesCmd.command("show").description("\u67E5\u770B\u7B14\u8BB0\u8BE6\u60C5").argument("<id>", "\u7B14\u8BB0ID").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((id, options) => {
  try {
    const note = getDatabase().prepare("SELECT * FROM notes WHERE id = ?").get(id);
    if (!note) {
      console.log(source_default.red("\u7B14\u8BB0\u4E0D\u5B58\u5728"));
      return;
    }
    if (options.json) {
      console.log(JSON.stringify(note, null, 2));
      return;
    }
    console.log(source_default.bold.cyan(note.title || "\u65E0\u6807\u9898"));
    console.log(source_default.gray(`ID: ${note.id} | ${formatDate(note.updatedAt)}`));
    if (note.pinned) console.log(source_default.yellow("\u{1F4CC} \u5DF2\u7F6E\u9876"));
    if (note.groupId) console.log(source_default.magenta(`\u5206\u7EC4: ${note.groupId}`));
    console.log("\u2500".repeat(40));
    console.log(note.content || "(\u7A7A)");
  } catch (error) {
    console.error(source_default.red(`\u67E5\u770B\u7B14\u8BB0\u5931\u8D25: ${error.message}`));
  }
});
notesCmd.command("edit").description("\u7F16\u8F91\u7B14\u8BB0").argument("<id>", "\u7B14\u8BB0ID").option("-t, --title <title>", "\u65B0\u6807\u9898").option("-c, --content <text>", "\u65B0\u5185\u5BB9").option("-g, --group <id>", "\u6240\u5C5E\u5206\u7EC4").option("--pin", "\u7F6E\u9876").option("--unpin", "\u53D6\u6D88\u7F6E\u9876").action((id, options) => {
  try {
    const db2 = getDatabase();
    const existing = db2.prepare("SELECT * FROM notes WHERE id = ?").get(id);
    if (!existing) {
      console.log(source_default.red("\u7B14\u8BB0\u4E0D\u5B58\u5728"));
      return;
    }
    const updates = {};
    if (options.title) updates.title = options.title;
    if (options.content) updates.content = options.content;
    if (options.group !== void 0) updates.groupId = options.group;
    if (options.pin) updates.pinned = 1;
    if (options.unpin) updates.pinned = 0;
    updates.updatedAt = (/* @__PURE__ */ new Date()).toISOString();
    const fields = Object.keys(updates).map((k2) => `${k2} = ?`).join(", ");
    const values = [...Object.values(updates), id];
    db2.prepare(`UPDATE notes SET ${fields} WHERE id = ?`).run(...values);
    console.log(source_default.green("\u2713 \u7B14\u8BB0\u5DF2\u66F4\u65B0"));
  } catch (error) {
    console.error(source_default.red(`\u7F16\u8F91\u7B14\u8BB0\u5931\u8D25: ${error.message}`));
  }
});
notesCmd.command("delete").alias("del").description("\u5220\u9664\u7B14\u8BB0").argument("<id>", "\u7B14\u8BB0ID").action((id) => {
  try {
    getDatabase().prepare("DELETE FROM notes WHERE id = ?").run(id);
    console.log(source_default.green("\u2713 \u7B14\u8BB0\u5DF2\u5220\u9664"));
  } catch (error) {
    console.error(source_default.red(`\u5220\u9664\u7B14\u8BB0\u5931\u8D25: ${error.message}`));
  }
});
notesCmd.command("search").description("\u641C\u7D22\u7B14\u8BB0").argument("<query>", "\u641C\u7D22\u5173\u952E\u8BCD").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((query, options) => {
  try {
    const notes = getDatabase().prepare(
      "SELECT * FROM notes WHERE title LIKE ? OR content LIKE ? ORDER BY pinned DESC, updatedAt DESC"
    ).all(`%${query}%`, `%${query}%`);
    printNotes(notes, options);
  } catch (error) {
    console.error(source_default.red(`\u641C\u7D22\u7B14\u8BB0\u5931\u8D25: ${error.message}`));
  }
});
var noteGroupCmd = notesCmd.command("groups").description("\u7B14\u8BB0\u5206\u7EC4\u7BA1\u7406");
noteGroupCmd.command("list").alias("ls").description("\u5217\u51FA\u7B14\u8BB0\u5206\u7EC4").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((options) => {
  try {
    const groups = getDatabase().prepare("SELECT * FROM note_groups ORDER BY sortOrder ASC, createdAt ASC").all();
    if (options.json) {
      console.log(JSON.stringify(groups, null, 2));
      return;
    }
    if (groups.length === 0) {
      console.log(source_default.gray("\u6682\u65E0\u5206\u7EC4"));
      return;
    }
    console.log(source_default.underline("\u7B14\u8BB0\u5206\u7EC4"));
    groups.forEach((g) => {
      console.log(`  ${g.icon || "\u{1F4C1}"} ${source_default.bold(g.name)} (${g.id})`);
    });
  } catch (error) {
    console.error(source_default.red(`\u5217\u51FA\u5206\u7EC4\u5931\u8D25: ${error.message}`));
  }
});
noteGroupCmd.command("add").description("\u65B0\u5EFA\u5206\u7EC4").argument("<name>", "\u5206\u7EC4\u540D\u79F0").option("-i, --icon <icon>", "\u5206\u7EC4\u56FE\u6807 (emoji)", "\u{1F4C1}").action((name, options) => {
  try {
    const id = `ng_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 6)}`;
    getDatabase().prepare("INSERT INTO note_groups (id, name, icon, sortOrder, createdAt) VALUES (?, ?, ?, 0, ?)").run(id, name, options.icon, (/* @__PURE__ */ new Date()).toISOString());
    console.log(source_default.green(`\u2713 \u5206\u7EC4\u5DF2\u521B\u5EFA: ${options.icon} ${name}`));
  } catch (error) {
    console.error(source_default.red(`\u521B\u5EFA\u5206\u7EC4\u5931\u8D25: ${error.message}`));
  }
});
noteGroupCmd.command("delete").alias("del").description("\u5220\u9664\u5206\u7EC4").argument("<id>", "\u5206\u7EC4ID").action((id) => {
  try {
    getDatabase().prepare("DELETE FROM note_groups WHERE id = ?").run(id);
    getDatabase().prepare("UPDATE notes SET groupId = NULL WHERE groupId = ?").run(id);
    console.log(source_default.green("\u2713 \u5206\u7EC4\u5DF2\u5220\u9664\uFF0C\u7B14\u8BB0\u5DF2\u79FB\u81F3\u672A\u5206\u7EC4"));
  } catch (error) {
    console.error(source_default.red(`\u5220\u9664\u5206\u7EC4\u5931\u8D25: ${error.message}`));
  }
});
var mfaCmd = program.command("mfa").alias("totp").description("MFA/TOTP \u9A8C\u8BC1\u7801\u7BA1\u7406");
mfaCmd.command("list").alias("ls").description("\u5217\u51FA\u6240\u6709MFA\u5BC6\u94A5").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((options) => {
  try {
    const secrets = getDatabase().prepare("SELECT * FROM mfa_secrets ORDER BY createdAt ASC").all();
    if (options.json) {
      console.log(JSON.stringify(secrets, null, 2));
      return;
    }
    if (secrets.length === 0) {
      console.log(source_default.gray("\u6682\u65E0MFA\u5BC6\u94A5"));
      return;
    }
    console.log(source_default.underline("MFA \u9A8C\u8BC1\u7801"));
    secrets.forEach((s) => {
      console.log(`  ${source_default.bold(s.name)} (${s.issuer || "\u9ED8\u8BA4"}) - ${s.algorithm.toUpperCase()} ${s.digits}\u4F4D`);
    });
    console.log(source_default.gray('\n\u4F7F\u7528 "todo mfa code <name>" \u67E5\u770B\u5F53\u524D\u9A8C\u8BC1\u7801'));
  } catch (error) {
    console.error(source_default.red(`\u5217\u51FAMFA\u5BC6\u94A5\u5931\u8D25: ${error.message}`));
  }
});
mfaCmd.command("add").description("\u6DFB\u52A0MFA\u5BC6\u94A5").argument("<name>", "\u540D\u79F0").argument("<secret>", "Base32\u5BC6\u94A5").option("-i, --issuer <issuer>", "\u53D1\u884C\u65B9").option("-a, --account <account>", "\u8D26\u53F7").option("-d, --digits <n>", "\u9A8C\u8BC1\u7801\u4F4D\u6570", "6").option("-p, --period <n>", "\u5237\u65B0\u95F4\u9694(\u79D2)", "30").option("-u, --uri <otpauth_uri>", "otpauth:// URI (\u81EA\u52A8\u89E3\u6790)").action(async (name, secret, options) => {
  try {
    let parsed = { secret, digits: parseInt(options.digits), period: parseInt(options.period), algorithm: "sha1", account: options.account || "", issuer: options.issuer || "" };
    if (options.uri) {
      parsed = parseOtpauthUri(options.uri);
      name = name || parsed.name;
    }
    const id = `mfa_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`;
    const now = (/* @__PURE__ */ new Date()).toISOString();
    getDatabase().prepare(
      "INSERT INTO mfa_secrets (id, name, secret, digits, period, algorithm, account, issuer, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    ).run(id, name, parsed.secret.toUpperCase(), parsed.digits, parsed.period, parsed.algorithm, parsed.account, parsed.issuer, now, now);
    console.log(source_default.green(`\u2713 MFA\u5BC6\u94A5\u5DF2\u6DFB\u52A0: ${name}`));
  } catch (error) {
    console.error(source_default.red(`\u6DFB\u52A0MFA\u5BC6\u94A5\u5931\u8D25: ${error.message}`));
  }
});
mfaCmd.command("code").description("\u751F\u6210\u5F53\u524D\u9A8C\u8BC1\u7801").argument("<name>", "\u5BC6\u94A5\u540D\u79F0").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((name, options) => {
  try {
    const { generateTOTP, getRemainingTime } = requireTotpModule();
    const db2 = getDatabase();
    const record = db2.prepare("SELECT * FROM mfa_secrets WHERE name LIKE ? OR issuer LIKE ? OR account LIKE ?").get(`%${name}%`, `%${name}%`, `%${name}%`);
    if (!record) {
      console.log(source_default.red(`\u672A\u627E\u5230MFA\u5BC6\u94A5: ${name}`));
      return;
    }
    const code = generateTOTP(record.secret, { digits: record.digits, period: record.period, algorithm: record.algorithm });
    const remaining = getRemainingTime(record.period);
    if (options.json) {
      console.log(JSON.stringify({ name: record.name, code, remaining }, null, 2));
    } else {
      console.log(source_default.bold.cyan(`${record.name}: ${code}`));
      console.log(source_default.gray(`\u5269\u4F59 ${remaining} \u79D2`));
    }
  } catch (error) {
    console.error(source_default.red(`\u751F\u6210\u9A8C\u8BC1\u7801\u5931\u8D25: ${error.message}`));
  }
});
mfaCmd.command("delete").alias("del").description("\u5220\u9664MFA\u5BC6\u94A5").argument("<name>", "\u5BC6\u94A5\u540D\u79F0").action((name) => {
  try {
    const db2 = getDatabase();
    const record = db2.prepare("SELECT * FROM mfa_secrets WHERE name = ?").get(name);
    if (!record) {
      console.log(source_default.red(`\u672A\u627E\u5230MFA\u5BC6\u94A5: ${name}`));
      return;
    }
    db2.prepare("DELETE FROM mfa_secrets WHERE name = ?").run(name);
    console.log(source_default.green(`\u2713 MFA\u5BC6\u94A5\u5DF2\u5220\u9664: ${name}`));
  } catch (error) {
    console.error(source_default.red(`\u5220\u9664MFA\u5BC6\u94A5\u5931\u8D25: ${error.message}`));
  }
});
var gitSyncCmd = program.command("git-sync").alias("gsync").description("Git \u6570\u636E\u540C\u6B65");
gitSyncCmd.command("status").description("\u67E5\u770B\u540C\u6B65\u72B6\u6001").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action(async (options) => {
  try {
    const db2 = getDatabase();
    const enabled = db2.prepare("SELECT value FROM settings WHERE key = 'git_sync_enabled'").get();
    const remoteUrl = db2.prepare("SELECT value FROM settings WHERE key = 'git_sync_remote_url'").get();
    const branch = db2.prepare("SELECT value FROM settings WHERE key = 'git_sync_branch'").get();
    const interval = db2.prepare("SELECT value FROM settings WHERE key = 'git_sync_interval'").get();
    const lastSync = db2.prepare("SELECT value FROM settings WHERE key = 'git_sync_last_sync'").get();
    const status = db2.prepare("SELECT value FROM settings WHERE key = 'git_sync_status'").get();
    const error = db2.prepare("SELECT value FROM settings WHERE key = 'git_sync_error'").get();
    const info = {
      enabled: enabled?.value === "true",
      remoteUrl: remoteUrl?.value || null,
      branch: branch?.value || "main",
      interval: parseInt(interval?.value || "5"),
      lastSync: lastSync?.value || null,
      status: status?.value || "not_configured",
      error: error?.value || null
    };
    if (options.json) {
      console.log(JSON.stringify(info, null, 2));
      return;
    }
    console.log(source_default.underline("Git \u540C\u6B65\u72B6\u6001"));
    console.log(`  \u72B6\u6001: ${info.enabled ? source_default.green("\u5DF2\u542F\u7528") : source_default.gray("\u672A\u542F\u7528")}`);
    console.log(`  \u8FDC\u7A0B: ${info.remoteUrl || source_default.gray("\u672A\u914D\u7F6E")}`);
    console.log(`  \u5206\u652F: ${info.branch}`);
    console.log(`  \u95F4\u9694: ${info.interval} \u5206\u949F`);
    console.log(`  \u4E0A\u6B21\u540C\u6B65: ${info.lastSync ? formatDate(info.lastSync) : source_default.gray("\u4ECE\u672A")}`);
    console.log(`  \u72B6\u6001\u7801: ${info.status === "ok" ? source_default.green("\u6B63\u5E38") : source_default.red(info.status)}`);
    if (info.error) console.log(`  \u9519\u8BEF: ${source_default.red(info.error)}`);
  } catch (error) {
    console.error(source_default.red(`\u67E5\u770B\u540C\u6B65\u72B6\u6001\u5931\u8D25: ${error.message}`));
  }
});
gitSyncCmd.command("configure").description("\u914D\u7F6EGit\u540C\u6B65").option("-r, --remote <url>", "\u8FDC\u7A0B\u4ED3\u5E93\u5730\u5740").option("-b, --branch <branch>", "\u5206\u652F\u540D", "main").option("-i, --interval <minutes>", "\u540C\u6B65\u95F4\u9694(\u5206\u949F)", "5").option("--enable", "\u542F\u7528\u540C\u6B65").option("--disable", "\u7981\u7528\u540C\u6B65").action((options) => {
  try {
    const db2 = getDatabase();
    if (options.remote) db2.prepare("INSERT OR REPLACE INTO settings (key, value) VALUES ('git_sync_remote_url', ?)").run(options.remote);
    db2.prepare("INSERT OR REPLACE INTO settings (key, value) VALUES ('git_sync_branch', ?)").run(options.branch);
    db2.prepare("INSERT OR REPLACE INTO settings (key, value) VALUES ('git_sync_interval', ?)").run(options.interval);
    if (options.enable) db2.prepare("INSERT OR REPLACE INTO settings (key, value) VALUES ('git_sync_enabled', 'true')").run();
    if (options.disable) db2.prepare("INSERT OR REPLACE INTO settings (key, value) VALUES ('git_sync_enabled', 'false')").run();
    console.log(source_default.green("\u2713 Git\u540C\u6B65\u914D\u7F6E\u5DF2\u66F4\u65B0"));
  } catch (error) {
    console.error(source_default.red(`\u914D\u7F6E\u5931\u8D25: ${error.message}`));
  }
});
gitSyncCmd.command("sync").alias("push").description("\u624B\u52A8\u63A8\u9001\u6570\u636E\u5230Git\u8FDC\u7A0B\u4ED3\u5E93").action(async () => {
  try {
    const simpleGit2 = (await Promise.resolve().then(() => (init_esm(), esm_exports))).default;
    const db2 = getDatabase();
    const remoteUrl = db2.prepare("SELECT value FROM settings WHERE key = 'git_sync_remote_url'").get();
    if (!remoteUrl) {
      console.log(source_default.red("\u672A\u914D\u7F6E\u8FDC\u7A0B\u4ED3\u5E93\uFF0C\u8BF7\u5148\u8FD0\u884C: todo git-sync configure -r <url>"));
      return;
    }
    const syncPath = path.join(os2.homedir(), ".supertool", "sync-data");
    if (!fs.existsSync(syncPath)) fs.mkdirSync(syncPath, { recursive: true });
    const git = simpleGit2(syncPath);
    const isRepo = fs.existsSync(path.join(syncPath, ".git"));
    if (!isRepo) {
      console.log(source_default.yellow("\u521D\u59CB\u5316Git\u4ED3\u5E93..."));
      await git.init();
      try {
        await git.addRemote("origin", remoteUrl.value);
      } catch {
      }
    }
    const allData = exportAllDataFromDb();
    fs.writeFileSync(path.join(syncPath, "data.json"), JSON.stringify(allData, null, 2));
    await git.add("data.json");
    const status = await git.status();
    if (status.staged.length === 0 && status.conflicted.length === 0) {
      console.log(source_default.green("\u2713 \u6570\u636E\u5DF2\u662F\u6700\u65B0\uFF0C\u65E0\u9700\u540C\u6B65"));
      return;
    }
    await git.commit(`CLI sync: ${(/* @__PURE__ */ new Date()).toISOString()}`);
    console.log(source_default.yellow("\u6B63\u5728\u63A8\u9001..."));
    await git.push("origin", "main");
    db2.prepare("INSERT OR REPLACE INTO settings (key, value) VALUES ('git_sync_last_sync', ?)").run((/* @__PURE__ */ new Date()).toISOString());
    db2.prepare("INSERT OR REPLACE INTO settings (key, value) VALUES ('git_sync_status', 'ok')").run();
    console.log(source_default.green("\u2713 \u6570\u636E\u5DF2\u63A8\u9001\u5230\u8FDC\u7A0B\u4ED3\u5E93"));
  } catch (error) {
    console.error(source_default.red(`\u540C\u6B65\u5931\u8D25: ${error.message}`));
  }
});
gitSyncCmd.command("pull").description("\u4ECEGit\u8FDC\u7A0B\u4ED3\u5E93\u62C9\u53D6\u6570\u636E").action(async () => {
  try {
    const simpleGit2 = (await Promise.resolve().then(() => (init_esm(), esm_exports))).default;
    const syncPath = path.join(os2.homedir(), ".supertool", "sync-data");
    if (!fs.existsSync(path.join(syncPath, ".git"))) {
      console.log(source_default.red("Git\u4ED3\u5E93\u672A\u521D\u59CB\u5316"));
      return;
    }
    const git = simpleGit2(syncPath);
    console.log(source_default.yellow("\u6B63\u5728\u62C9\u53D6..."));
    await git.pull("origin", "main");
    const data = JSON.parse(fs.readFileSync(path.join(syncPath, "data.json"), "utf8"));
    const result = importDataToDb(data, "merge");
    console.log(source_default.green(`\u2713 \u62C9\u53D6\u6210\u529F\uFF0C\u5BFC\u5165 ${result.imported} \u6761\uFF0C\u8DF3\u8FC7 ${result.skipped} \u6761`));
  } catch (error) {
    console.error(source_default.red(`\u62C9\u53D6\u5931\u8D25: ${error.message}`));
  }
});
var serverGroupCmd = program.command("server-group").alias("sg").description("\u670D\u52A1\u5668\u5206\u7EC4\u7BA1\u7406");
serverGroupCmd.command("list").alias("ls").description("\u5217\u51FA\u670D\u52A1\u5668\u5206\u7EC4").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((options) => {
  try {
    const groups = getDatabase().prepare("SELECT * FROM server_groups ORDER BY createdAt DESC").all();
    if (options.json) {
      console.log(JSON.stringify(groups, null, 2));
      return;
    }
    if (groups.length === 0) {
      console.log(source_default.gray("\u6682\u65E0\u5206\u7EC4"));
      return;
    }
    console.log(source_default.underline("\u670D\u52A1\u5668\u5206\u7EC4"));
    groups.forEach((g) => {
      console.log(`  ${source_default.bold(g.name)} - ${g.description || "\u65E0\u63CF\u8FF0"} (${g.id})`);
    });
  } catch (error) {
    console.error(source_default.red(`\u5217\u51FA\u5206\u7EC4\u5931\u8D25: ${error.message}`));
  }
});
serverGroupCmd.command("add").description("\u65B0\u5EFA\u670D\u52A1\u5668\u5206\u7EC4").argument("<name>", "\u5206\u7EC4\u540D\u79F0").option("-d, --description <desc>", "\u63CF\u8FF0").action((name, options) => {
  try {
    const id = `sg_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 6)}`;
    const now = (/* @__PURE__ */ new Date()).toISOString();
    getDatabase().prepare("INSERT INTO server_groups (id, name, description, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?)").run(id, name, options.description || "", now, now);
    console.log(source_default.green(`\u2713 \u5206\u7EC4\u5DF2\u521B\u5EFA: ${name}`));
  } catch (error) {
    console.error(source_default.red(`\u521B\u5EFA\u5206\u7EC4\u5931\u8D25: ${error.message}`));
  }
});
serverGroupCmd.command("delete").alias("del").description("\u5220\u9664\u670D\u52A1\u5668\u5206\u7EC4").argument("<id>", "\u5206\u7EC4ID").action((id) => {
  try {
    getDatabase().prepare("DELETE FROM server_groups WHERE id = ?").run(id);
    console.log(source_default.green("\u2713 \u5206\u7EC4\u5DF2\u5220\u9664"));
  } catch (error) {
    console.error(source_default.red(`\u5220\u9664\u5206\u7EC4\u5931\u8D25: ${error.message}`));
  }
});
var deployHistCmd = program.command("deploy-history").alias("dh").description("\u90E8\u7F72\u5386\u53F2\u7BA1\u7406");
deployHistCmd.command("list").alias("ls").description("\u5217\u51FA\u90E8\u7F72\u5386\u53F2").option("-p, --project <projectId>", "\u9879\u76EEID\u8FC7\u6EE4").option("-l, --limit <n>", "\u663E\u793A\u6570\u91CF", "20").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((options) => {
  try {
    const db2 = getDatabase();
    let sql = "SELECT * FROM deploy_history ORDER BY deployedAt DESC LIMIT ?";
    let params = [parseInt(options.limit)];
    if (options.project) {
      sql = "SELECT * FROM deploy_history WHERE projectId = ? ORDER BY deployedAt DESC LIMIT ?";
      params = [options.project, parseInt(options.limit)];
    }
    const history = db2.prepare(sql).all(...params);
    if (options.json) {
      console.log(JSON.stringify(history, null, 2));
      return;
    }
    if (history.length === 0) {
      console.log(source_default.gray("\u6682\u65E0\u90E8\u7F72\u5386\u53F2"));
      return;
    }
    console.log(source_default.underline("\u90E8\u7F72\u5386\u53F2"));
    history.forEach((h2, i2) => {
      const status = h2.status === "success" ? source_default.green("\u2713") : source_default.red("\u2717");
      const rollback = h2.rolledBack ? source_default.yellow(" [\u5DF2\u56DE\u6EDA]") : "";
      console.log(`  ${status} ${h2.version || "N/A"} - ${h2.projectId}${rollback}`);
      console.log(`     ${formatDate(h2.deployedAt)} | commit: ${h2.gitCommit || "N/A"}`);
    });
  } catch (error) {
    console.error(source_default.red(`\u5217\u51FA\u90E8\u7F72\u5386\u53F2\u5931\u8D25: ${error.message}`));
  }
});
deployHistCmd.command("rollback").description("\u56DE\u6EDA\u5230\u6307\u5B9A\u7248\u672C").argument("<deployHistoryId>", "\u90E8\u7F72\u5386\u53F2ID").action((id) => {
  try {
    const db2 = getDatabase();
    const record = db2.prepare("SELECT * FROM deploy_history WHERE id = ?").get(id);
    if (!record) {
      console.log(source_default.red("\u90E8\u7F72\u8BB0\u5F55\u4E0D\u5B58\u5728"));
      return;
    }
    db2.prepare("UPDATE deploy_history SET rolledBack = 1, rolledBackAt = ? WHERE id = ?").run((/* @__PURE__ */ new Date()).toISOString(), id);
    console.log(source_default.green(`\u2713 \u5DF2\u6807\u8BB0\u56DE\u6EDA: ${record.version || id}`));
  } catch (error) {
    console.error(source_default.red(`\u56DE\u6EDA\u5931\u8D25: ${error.message}`));
  }
});
var notifCmd = program.command("notification").alias("notif").description("\u901A\u77E5\u7BA1\u7406");
notifCmd.command("list").alias("ls").description("\u5217\u51FA\u901A\u77E5").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((options) => {
  try {
    const db2 = getDatabase();
    const reminderTime = db2.getSetting?.("reminder_time") || "15";
    if (options.json) {
      console.log(JSON.stringify({ reminderTime: parseInt(reminderTime) }, null, 2));
      return;
    }
    console.log(source_default.underline("\u901A\u77E5\u8BBE\u7F6E"));
    console.log(`  \u63D0\u9192\u65F6\u95F4: \u622A\u6B62\u65E5\u671F\u524D ${reminderTime} \u5206\u949F`);
  } catch (error) {
    console.error(source_default.red(`\u83B7\u53D6\u901A\u77E5\u8BBE\u7F6E\u5931\u8D25: ${error.message}`));
  }
});
notifCmd.command("config").description("\u914D\u7F6E\u901A\u77E5\u8BBE\u7F6E").option("-t, --time <minutes>", "\u63D0\u524D\u63D0\u9192\u65F6\u95F4(\u5206\u949F)", "15").action((options) => {
  try {
    const db2 = getDatabase();
    if (db2.setSetting) {
      db2.setSetting("reminder_time", options.time);
    } else {
      db2.prepare("INSERT OR REPLACE INTO settings (key, value) VALUES ('reminder_time', ?)").run(options.time);
    }
    console.log(source_default.green(`\u2713 \u901A\u77E5\u63D0\u9192\u5DF2\u8BBE\u7F6E\u4E3A\u622A\u6B62\u65E5\u671F\u524D ${options.time} \u5206\u949F`));
  } catch (error) {
    console.error(source_default.red(`\u914D\u7F6E\u901A\u77E5\u5931\u8D25: ${error.message}`));
  }
});
program.command("guide").alias("man").description("\u67E5\u770B CLI \u4F7F\u7528\u6307\u5357").option("-j, --json", "\u8F93\u51FA JSON \u683C\u5F0F").action((options) => {
  const possiblePaths = [
    // PKG 安装路径 (嵌入在 app bundle 内)
    path.join(__dirname, "docs", "cli-guide.md"),
    // 开发/源码路径
    path.join(__dirname, "..", "docs", "cli-guide.md"),
    // Linux 安装路径
    "/opt/SuperTool/resources/docs/cli-guide.md"
  ];
  let guidePath = null;
  for (const p2 of possiblePaths) {
    if (fs.existsSync(p2)) {
      guidePath = p2;
      break;
    }
  }
  if (!guidePath) {
    console.error(source_default.red("\u9519\u8BEF: \u627E\u4E0D\u5230\u4F7F\u7528\u6307\u5357\u6587\u4EF6"));
    console.error(source_default.gray("\u5C1D\u8BD5\u7684\u8DEF\u5F84:"));
    possiblePaths.forEach((p2) => console.error(source_default.gray(`  - ${p2}`)));
    return;
  }
  if (options.json) {
    const content2 = fs.readFileSync(guidePath, "utf-8");
    console.log(JSON.stringify({ guide: content2, path: guidePath }));
    return;
  }
  const content = fs.readFileSync(guidePath, "utf-8");
  const lines = content.split("\n");
  for (const line of lines) {
    if (line.startsWith("### ")) {
      console.log(source_default.bold.cyan(line.slice(4)));
    } else if (line.startsWith("## ")) {
      console.log(source_default.bold.green("\n" + line.slice(3)));
    } else if (line.startsWith("# ")) {
      console.log(source_default.bold.underline.green("\n" + line.slice(2) + "\n"));
    } else if (line.startsWith("```")) {
      console.log(source_default.gray(line));
    } else if (line.startsWith("> ")) {
      console.log(source_default.dim("  " + line.slice(2)));
    } else if (line.startsWith("- ") || line.startsWith("* ")) {
      console.log(source_default.yellow("  " + line));
    } else if (line.trim()) {
      console.log(line);
    } else {
      console.log();
    }
  }
  console.log();
});
var accountingCmd = program.command("accounting").alias("acct").description("\u8BB0\u8D26\u7BA1\u7406");
function formatAmount(amount, type) {
  const formatted = `\xA5${parseFloat(amount).toFixed(2)}`;
  return type === "expense" ? source_default.red(formatted) : source_default.green(formatted);
}
accountingCmd.command("list").alias("ls").description("\u5217\u51FA\u6700\u8FD1\u8BB0\u8D26\u8BB0\u5F55").option("-t, --type <type>", "\u7C7B\u578B: expense|income").option("-c, --category <category>", "\u5206\u7C7B\u8FC7\u6EE4").option("-l, --limit <n>", "\u663E\u793A\u6570\u91CF", "20").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((options) => {
  try {
    const db2 = getDatabase();
    let sql = "SELECT * FROM accounting_records WHERE 1=1";
    const params = [];
    if (options.type) {
      sql += " AND type = ?";
      params.push(options.type);
    }
    if (options.category) {
      sql += " AND category = ?";
      params.push(options.category);
    }
    sql += " ORDER BY date DESC LIMIT ?";
    params.push(parseInt(options.limit));
    const records = db2.prepare(sql).all(...params).map((r2) => {
      try {
        r2.attachments_json = JSON.parse(r2.attachments_json || "[]");
      } catch {
        r2.attachments_json = [];
      }
      return r2;
    });
    if (options.json) {
      console.log(JSON.stringify(records, null, 2));
      return;
    }
    if (records.length === 0) {
      console.log(source_default.gray("\u6682\u65E0\u8BB0\u8D26\u8BB0\u5F55"));
      return;
    }
    console.log(source_default.underline("\u8BB0\u8D26\u8BB0\u5F55"));
    records.forEach((r2) => {
      const typeIcon = r2.type === "expense" ? "\u{1F53B}" : "\u{1F53A}";
      const typeLabel = r2.type === "expense" ? "\u652F\u51FA" : "\u6536\u5165";
      const cat = r2.category ? source_default.magenta(` [${r2.category}]`) : "";
      const desc = r2.description ? source_default.gray(` - ${r2.description}`) : "";
      console.log(`  ${typeIcon} ${formatAmount(r2.amount, r2.type)}${cat}${desc}`);
      console.log(`     ${source_default.gray(formatDate(r2.date))} (#${r2.id})`);
    });
  } catch (error) {
    console.error(source_default.red(`\u5217\u51FA\u8BB0\u5F55\u5931\u8D25: ${error.message}`));
  }
});
accountingCmd.command("add").description("\u65B0\u589E\u8BB0\u8D26\u8BB0\u5F55").argument("<amount>", "\u91D1\u989D").option("-c, --category <category>", "\u5206\u7C7B", "\u672A\u5206\u7C7B").option("-t, --type <type>", "\u7C7B\u578B: expense|income", "expense").option("-d, --desc <description>", "\u63CF\u8FF0", "").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((amount, options) => {
  try {
    const amt = parseFloat(amount);
    if (isNaN(amt) || amt <= 0) {
      console.log(source_default.red("\u91D1\u989D\u5FC5\u987B\u4E3A\u6B63\u6570"));
      return;
    }
    if (!["expense", "income"].includes(options.type)) {
      console.log(source_default.red("\u7C7B\u578B\u5FC5\u987B\u662F expense \u6216 income"));
      return;
    }
    const db2 = getDatabase();
    const now = (/* @__PURE__ */ new Date()).toISOString();
    const id = `ar_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`;
    try {
      db2.exec("ALTER TABLE accounting_records ADD COLUMN attachments_json TEXT DEFAULT '[]'");
    } catch {
    }
    const result = db2.prepare(
      "INSERT INTO accounting_records (id, date, type, category, amount, description, status, createdAt, updatedAt, attachments_json) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    ).run(id, now, options.type, options.category, amt, options.desc, "completed", now, now, "[]");
    const record = db2.prepare("SELECT * FROM accounting_records WHERE id = ?").get(id);
    try {
      record.attachments_json = JSON.parse(record.attachments_json || "[]");
    } catch {
      record.attachments_json = [];
    }
    if (options.json) {
      console.log(JSON.stringify(record, null, 2));
      return;
    }
    console.log(source_default.green(`\u2713 \u8BB0\u5F55\u5DF2\u6DFB\u52A0: ${formatAmount(amt, options.type)} [${options.category}]${options.desc ? " - " + options.desc : ""}`));
  } catch (error) {
    console.error(source_default.red(`\u6DFB\u52A0\u8BB0\u5F55\u5931\u8D25: ${error.message}`));
  }
});
accountingCmd.command("show").description("\u67E5\u770B\u8BB0\u5F55\u8BE6\u60C5").argument("<id>", "\u8BB0\u5F55ID").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((id, options) => {
  try {
    const db2 = getDatabase();
    const record = db2.prepare("SELECT * FROM accounting_records WHERE id = ?").get(id);
    if (!record) {
      console.log(source_default.red(`\u8BB0\u5F55\u4E0D\u5B58\u5728: ${id}`));
      return;
    }
    try {
      record.attachments_json = JSON.parse(record.attachments_json || "[]");
    } catch {
      record.attachments_json = [];
    }
    if (options.json) {
      console.log(JSON.stringify(record, null, 2));
      return;
    }
    const typeLabel = record.type === "expense" ? "\u652F\u51FA" : "\u6536\u5165";
    console.log(source_default.bold(`#${record.id} ${typeLabel}`));
    console.log(`  \u91D1\u989D: ${formatAmount(record.amount, record.type)}`);
    console.log(`  \u5206\u7C7B: ${source_default.magenta(record.category || "\u672A\u5206\u7C7B")}`);
    if (record.description) console.log(`  \u63CF\u8FF0: ${record.description}`);
    console.log(`  \u65F6\u95F4: ${formatDate(record.date)}`);
    if (record.attachments_json && record.attachments_json.length > 0) {
      console.log(`  \u9644\u4EF6: ${record.attachments_json.length} \u4E2A`);
    }
  } catch (error) {
    console.error(source_default.red(`\u67E5\u770B\u8BB0\u5F55\u5931\u8D25: ${error.message}`));
  }
});
accountingCmd.command("delete").alias("del").description("\u5220\u9664\u8BB0\u8D26\u8BB0\u5F55").argument("<id>", "\u8BB0\u5F55ID").action((id) => {
  try {
    const db2 = getDatabase();
    const record = db2.prepare("SELECT * FROM accounting_records WHERE id = ?").get(id);
    if (!record) {
      console.log(source_default.red(`\u8BB0\u5F55\u4E0D\u5B58\u5728: ${id}`));
      return;
    }
    db2.prepare("DELETE FROM accounting_records WHERE id = ?").run(id);
    console.log(source_default.green(`\u2713 \u8BB0\u5F55\u5DF2\u5220\u9664: ${formatAmount(record.amount, record.type)} [${record.category}]`));
  } catch (error) {
    console.error(source_default.red(`\u5220\u9664\u8BB0\u5F55\u5931\u8D25: ${error.message}`));
  }
});
accountingCmd.command("stats").description("\u7EDF\u8BA1").option("-p, --period <period>", "\u7EDF\u8BA1\u5468\u671F: month|quarter|year", "month").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((options) => {
  try {
    const db2 = getDatabase();
    const now = /* @__PURE__ */ new Date();
    let startDate;
    switch (options.period) {
      case "month":
        startDate = new Date(now.getFullYear(), now.getMonth(), 1).toISOString();
        break;
      case "quarter":
        const quarter = Math.floor(now.getMonth() / 3);
        startDate = new Date(now.getFullYear(), quarter * 3, 1).toISOString();
        break;
      case "year":
        startDate = new Date(now.getFullYear(), 0, 1).toISOString();
        break;
      default:
        startDate = new Date(now.getFullYear(), now.getMonth(), 1).toISOString();
    }
    const expenseTotal = db2.prepare(
      "SELECT COALESCE(SUM(amount), 0) as total FROM accounting_records WHERE type = ? AND date >= ?"
    ).get("expense", startDate).total;
    const incomeTotal = db2.prepare(
      "SELECT COALESCE(SUM(amount), 0) as total FROM accounting_records WHERE type = ? AND date >= ?"
    ).get("income", startDate).total;
    const expenseCount = db2.prepare(
      "SELECT COUNT(*) as count FROM accounting_records WHERE type = ? AND date >= ?"
    ).get("expense", startDate).count;
    const incomeCount = db2.prepare(
      "SELECT COUNT(*) as count FROM accounting_records WHERE type = ? AND date >= ?"
    ).get("income", startDate).count;
    const byCategory = db2.prepare(
      "SELECT category, SUM(amount) as total, COUNT(*) as count FROM accounting_records WHERE date >= ? GROUP BY category ORDER BY total DESC"
    ).all(startDate);
    const stats = {
      period: options.period,
      startDate,
      expense: { total: parseFloat(expenseTotal), count: expenseCount },
      income: { total: parseFloat(incomeTotal), count: incomeCount },
      balance: parseFloat(incomeTotal) - parseFloat(expenseTotal),
      byCategory
    };
    if (options.json) {
      console.log(JSON.stringify(stats, null, 2));
      return;
    }
    const periodLabel = { month: "\u672C\u6708", quarter: "\u672C\u5B63\u5EA6", year: "\u672C\u5E74\u5EA6" }[options.period] || "\u672C\u6708";
    console.log(source_default.underline(`\u8BB0\u8D26\u7EDF\u8BA1 - ${periodLabel}`));
    console.log(`  \u652F\u51FA: ${source_default.red(`\xA5${parseFloat(expenseTotal).toFixed(2)}`)} (${expenseCount} \u7B14)`);
    console.log(`  \u6536\u5165: ${source_default.green(`\xA5${parseFloat(incomeTotal).toFixed(2)}`)} (${incomeCount} \u7B14)`);
    console.log(`  \u7ED3\u4F59: ${(stats.balance >= 0 ? source_default.green : source_default.red)(`\xA5${stats.balance.toFixed(2)}`)}`);
    if (byCategory.length > 0) {
      console.log(source_default.gray("\n  \u6309\u5206\u7C7B:"));
      byCategory.forEach((c3) => {
        console.log(`    ${source_default.bold(c3.category)}: \xA5${parseFloat(c3.total).toFixed(2)} (${c3.count}\u7B14)`);
      });
    }
  } catch (error) {
    console.error(source_default.red(`\u7EDF\u8BA1\u5931\u8D25: ${error.message}`));
  }
});
var budgetCmd = accountingCmd.command("budget").description("\u9884\u7B97\u7BA1\u7406");
budgetCmd.command("list").alias("ls").description("\u5217\u51FA\u9884\u7B97").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((options) => {
  try {
    const db2 = getDatabase();
    const budgets = db2.prepare("SELECT * FROM accounting_budgets ORDER BY createdAt DESC").all();
    if (options.json) {
      console.log(JSON.stringify(budgets, null, 2));
      return;
    }
    if (budgets.length === 0) {
      console.log(source_default.gray("\u6682\u65E0\u9884\u7B97"));
      return;
    }
    console.log(source_default.underline("\u9884\u7B97"));
    budgets.forEach((b2) => {
      const periodBase = (b2.period || "").replace("ly", "").replace("rly", "r");
      const periodLabel = { month: "\u6708", quarter: "\u5B63", year: "\u5E74" }[periodBase] || b2.period;
      const now = /* @__PURE__ */ new Date();
      let startDate;
      if (periodBase === "month") startDate = new Date(now.getFullYear(), now.getMonth(), 1).toISOString();
      else if (periodBase === "quarter") {
        const q2 = Math.floor(now.getMonth() / 3);
        startDate = new Date(now.getFullYear(), q2 * 3, 1).toISOString();
      } else startDate = new Date(now.getFullYear(), 0, 1).toISOString();
      const spent = db2.prepare(
        "SELECT COALESCE(SUM(amount), 0) as total FROM accounting_records WHERE type = ? AND category = ? AND date >= ?"
      ).get("expense", b2.category, startDate).total;
      const remaining = b2.amount - spent;
      const pct = (spent / b2.amount * 100).toFixed(0);
      const color = remaining < 0 ? source_default.red : remaining < b2.amount * 0.2 ? source_default.yellow : source_default.green;
      console.log(`  ${source_default.bold(b2.category)}: ${formatAmount(b2.amount, "expense")}/${periodLabel} | \u5DF2\u7528 ${formatAmount(spent, "expense")} (${pct}%) | \u5269\u4F59 ${color(`\xA5${remaining.toFixed(2)}`)} (#${b2.id})`);
    });
  } catch (error) {
    console.error(source_default.red(`\u5217\u51FA\u9884\u7B97\u5931\u8D25: ${error.message}`));
  }
});
budgetCmd.command("add").description("\u6DFB\u52A0\u9884\u7B97").argument("<category>", "\u5206\u7C7B").argument("<amount>", "\u91D1\u989D").option("-p, --period <period>", "\u5468\u671F: monthly|quarterly|yearly", "monthly").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((category, amount, options) => {
  try {
    const amt = parseFloat(amount);
    if (isNaN(amt) || amt <= 0) {
      console.log(source_default.red("\u91D1\u989D\u5FC5\u987B\u4E3A\u6B63\u6570"));
      return;
    }
    const db2 = getDatabase();
    const now = (/* @__PURE__ */ new Date()).toISOString();
    const id = `ab_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`;
    const result = db2.prepare(
      "INSERT INTO accounting_budgets (id, category, amount, period, createdAt) VALUES (?, ?, ?, ?, ?)"
    ).run(id, category, amt, options.period, now);
    const budget = db2.prepare("SELECT * FROM accounting_budgets WHERE id = ?").get(id);
    if (options.json) {
      console.log(JSON.stringify(budget, null, 2));
      return;
    }
    const periodLabel = { month: "\u6708", quarter: "\u5B63", year: "\u5E74" }[options.period] || options.period;
    console.log(source_default.green(`\u2713 \u9884\u7B97\u5DF2\u6DFB\u52A0: ${category} ${formatAmount(amt, "expense")}/${periodLabel}`));
  } catch (error) {
    console.error(source_default.red(`\u6DFB\u52A0\u9884\u7B97\u5931\u8D25: ${error.message}`));
  }
});
budgetCmd.command("delete").alias("del").description("\u5220\u9664\u9884\u7B97").argument("<id>", "\u9884\u7B97ID").action((id) => {
  try {
    const db2 = getDatabase();
    const budget = db2.prepare("SELECT * FROM accounting_budgets WHERE id = ?").get(id);
    if (!budget) {
      console.log(source_default.red(`\u9884\u7B97\u4E0D\u5B58\u5728: ${id}`));
      return;
    }
    db2.prepare("DELETE FROM accounting_budgets WHERE id = ?").run(id);
    console.log(source_default.green(`\u2713 \u9884\u7B97\u5DF2\u5220\u9664: ${budget.category} (#${id})`));
  } catch (error) {
    console.error(source_default.red(`\u5220\u9664\u9884\u7B97\u5931\u8D25: ${error.message}`));
  }
});
var templateCmd = accountingCmd.command("template").description("\u6A21\u677F\u7BA1\u7406");
templateCmd.command("list").alias("ls").description("\u5217\u51FA\u6A21\u677F").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((options) => {
  try {
    const db2 = getDatabase();
    const templates = db2.prepare("SELECT * FROM accounting_templates ORDER BY createdAt DESC").all();
    if (options.json) {
      console.log(JSON.stringify(templates, null, 2));
      return;
    }
    if (templates.length === 0) {
      console.log(source_default.gray("\u6682\u65E0\u6A21\u677F"));
      return;
    }
    console.log(source_default.underline("\u8BB0\u8D26\u6A21\u677F"));
    templates.forEach((t2) => {
      const typeLabel = t2.type === "expense" ? "\u652F\u51FA" : "\u6536\u5165";
      const desc = t2.description ? source_default.gray(` - ${t2.description}`) : "";
      console.log(`  ${source_default.bold(t2.name)}: ${formatAmount(t2.amount, t2.type)} [${t2.category}] ${typeLabel}${desc} (#${t2.id})`);
    });
  } catch (error) {
    console.error(source_default.red(`\u5217\u51FA\u6A21\u677F\u5931\u8D25: ${error.message}`));
  }
});
templateCmd.command("add").description("\u6DFB\u52A0\u6A21\u677F").argument("<name>", "\u6A21\u677F\u540D\u79F0").option("-c, --category <category>", "\u5206\u7C7B", "\u672A\u5206\u7C7B").option("-t, --type <type>", "\u7C7B\u578B: expense|income", "expense").option("-a, --amount <amount>", "\u9ED8\u8BA4\u91D1\u989D", "0").option("-d, --desc <description>", "\u63CF\u8FF0", "").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((name, options) => {
  try {
    const amt = parseFloat(options.amount);
    if (isNaN(amt)) {
      console.log(source_default.red("\u91D1\u989D\u5FC5\u987B\u4E3A\u6570\u5B57"));
      return;
    }
    if (!["expense", "income"].includes(options.type)) {
      console.log(source_default.red("\u7C7B\u578B\u5FC5\u987B\u662F expense \u6216 income"));
      return;
    }
    const db2 = getDatabase();
    const now = (/* @__PURE__ */ new Date()).toISOString();
    const id = `at_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`;
    const result = db2.prepare(
      "INSERT INTO accounting_templates (id, name, category, type, amount, description, createdAt) VALUES (?, ?, ?, ?, ?, ?, ?)"
    ).run(id, name, options.category, options.type, amt, options.desc, now);
    const template = db2.prepare("SELECT * FROM accounting_templates WHERE id = ?").get(id);
    if (options.json) {
      console.log(JSON.stringify(template, null, 2));
      return;
    }
    console.log(source_default.green(`\u2713 \u6A21\u677F\u5DF2\u6DFB\u52A0: ${name} (${formatAmount(amt, options.type)} [${options.category}])`));
  } catch (error) {
    console.error(source_default.red(`\u6DFB\u52A0\u6A21\u677F\u5931\u8D25: ${error.message}`));
  }
});
templateCmd.command("use").description("\u4F7F\u7528\u6A21\u677F\uFF08\u6253\u5370\u6A21\u677F\u5185\u5BB9\uFF0C\u4E0D\u76F4\u63A5\u521B\u5EFA\u8BB0\u5F55\uFF09").argument("<id>", "\u6A21\u677FID").action((id) => {
  try {
    const db2 = getDatabase();
    const template = db2.prepare("SELECT * FROM accounting_templates WHERE id = ?").get(id);
    if (!template) {
      console.log(source_default.red(`\u6A21\u677F\u4E0D\u5B58\u5728: ${id}`));
      return;
    }
    const typeLabel = template.type === "expense" ? "\u652F\u51FA" : "\u6536\u5165";
    console.log(source_default.underline(`\u4F7F\u7528\u6A21\u677F: ${template.name}`));
    console.log(`  \u7C7B\u578B: ${typeLabel}`);
    console.log(`  \u5206\u7C7B: ${source_default.magenta(template.category)}`);
    console.log(`  \u91D1\u989D: ${formatAmount(template.amount, template.type)}`);
    if (template.description) console.log(`  \u63CF\u8FF0: ${template.description}`);
    console.log(source_default.gray("\n\u4F7F\u7528\u4EE5\u4E0B\u547D\u4EE4\u521B\u5EFA\u8BB0\u5F55:"));
    console.log(source_default.cyan(`  stool accounting add ${template.amount} -t ${template.type} -c "${template.category}" -d "${template.name}"`));
  } catch (error) {
    console.error(source_default.red(`\u4F7F\u7528\u6A21\u677F\u5931\u8D25: ${error.message}`));
  }
});
templateCmd.command("delete").alias("del").description("\u5220\u9664\u6A21\u677F").argument("<id>", "\u6A21\u677FID").action((id) => {
  try {
    const db2 = getDatabase();
    const template = db2.prepare("SELECT * FROM accounting_templates WHERE id = ?").get(id);
    if (!template) {
      console.log(source_default.red(`\u6A21\u677F\u4E0D\u5B58\u5728: ${id}`));
      return;
    }
    db2.prepare("DELETE FROM accounting_templates WHERE id = ?").run(id);
    console.log(source_default.green(`\u2713 \u6A21\u677F\u5DF2\u5220\u9664: ${template.name} (#${id})`));
  } catch (error) {
    console.error(source_default.red(`\u5220\u9664\u6A21\u677F\u5931\u8D25: ${error.message}`));
  }
});
accountingCmd.command("export").description("\u5BFC\u51FA\u4E3ACSV").option("-p, --period <period>", "\u5BFC\u51FA\u5468\u671F: month|quarter|year|all", "month").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((options) => {
  try {
    const db2 = getDatabase();
    let startDate;
    if (options.period !== "all") {
      const now = /* @__PURE__ */ new Date();
      switch (options.period) {
        case "month":
          startDate = new Date(now.getFullYear(), now.getMonth(), 1).toISOString();
          break;
        case "quarter": {
          const q2 = Math.floor(now.getMonth() / 3);
          startDate = new Date(now.getFullYear(), q2 * 3, 1).toISOString();
          break;
        }
        case "year":
          startDate = new Date(now.getFullYear(), 0, 1).toISOString();
          break;
        default:
          startDate = new Date(now.getFullYear(), now.getMonth(), 1).toISOString();
      }
    }
    let sql = "SELECT * FROM accounting_records";
    const params = [];
    if (startDate) {
      sql += " WHERE date >= ?";
      params.push(startDate);
    }
    sql += " ORDER BY date DESC";
    const records = db2.prepare(sql).all(...params);
    if (options.json) {
      const parsed = records.map((r2) => {
        try {
          r2.attachments_json = JSON.parse(r2.attachments_json || "[]");
        } catch {
          r2.attachments_json = [];
        }
        return r2;
      });
      console.log(JSON.stringify(parsed, null, 2));
      return;
    }
    if (records.length === 0) {
      console.log(source_default.gray("\u6682\u65E0\u53EF\u5BFC\u51FA\u7684\u8BB0\u5F55"));
      return;
    }
    const header = "ID,\u91D1\u989D,\u7C7B\u578B,\u5206\u7C7B,\u63CF\u8FF0,\u65F6\u95F4";
    const rows = records.map((r2) => {
      const desc = (r2.description || "").replace(/"/g, '""');
      return `${r2.id},${r2.amount},${r2.type === "expense" ? "\u652F\u51FA" : "\u6536\u5165"},"${r2.category}","${desc}",${r2.date}`;
    });
    const csv = [header, ...rows].join("\n");
    console.log(csv);
    console.error(source_default.green(`
\u2713 \u5DF2\u5BFC\u51FA ${records.length} \u6761\u8BB0\u5F55`));
  } catch (error) {
    console.error(source_default.red(`\u5BFC\u51FA\u5931\u8D25: ${error.message}`));
  }
});
var categoryCmd = accountingCmd.command("category").description("\u5206\u7C7B\u7BA1\u7406");
categoryCmd.command("list").alias("ls").description("\u5217\u51FA\u5206\u7C7B").option("-t, --type <type>", "\u7C7B\u578B\u8FC7\u6EE4: expense|income").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((options) => {
  try {
    const db2 = getDatabase();
    let sql = "SELECT * FROM accounting_categories";
    const params = [];
    if (options.type) {
      sql += " WHERE type = ?";
      params.push(options.type);
    }
    sql += " ORDER BY type, name";
    const categories = db2.prepare(sql).all(...params);
    if (options.json) {
      console.log(JSON.stringify(categories, null, 2));
      return;
    }
    if (categories.length === 0) {
      console.log(source_default.gray("\u6682\u65E0\u5206\u7C7B"));
      return;
    }
    console.log(source_default.underline("\u8BB0\u8D26\u5206\u7C7B"));
    let currentType = "";
    categories.forEach((c3) => {
      if (c3.type !== currentType) {
        currentType = c3.type;
        const label = currentType === "expense" ? source_default.red("\u652F\u51FA") : source_default.green("\u6536\u5165");
        console.log(`
  ${label}:`);
      }
      const icon = c3.icon ? c3.icon : "\u{1F4CC}";
      console.log(`    ${icon} ${source_default.bold(c3.name)}`);
    });
    const usedCategories = db2.prepare(
      'SELECT DISTINCT category, type FROM accounting_records WHERE category != "" AND category != ? ORDER BY category'
    ).all("\u672A\u5206\u7C7B");
    if (usedCategories.length > 0) {
      console.log(source_default.gray("\n  \u5DF2\u4F7F\u7528\u7684\u5206\u7C7B\uFF08\u672A\u5F55\u5165\u5206\u7C7B\u8868\uFF09:"));
      usedCategories.forEach((c3) => {
        console.log(`    ${c3.category} (${c3.type === "expense" ? "\u652F\u51FA" : "\u6536\u5165"})`);
      });
    }
  } catch (error) {
    console.error(source_default.red(`\u5217\u51FA\u5206\u7C7B\u5931\u8D25: ${error.message}`));
  }
});
program.command("version").alias("v").description("\u663E\u793A\u7248\u672C\u53F7").action(() => {
  console.log(source_default.green(`SuperTool CLI v${program.version()}`));
});
var gitCmd = program.command("git").description("Git \u4ED3\u5E93\u7BA1\u7406");
gitCmd.command("list").alias("ls").description("\u5217\u51FA\u5DF2\u4FDD\u5B58\u7684 Git \u4ED3\u5E93").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((options) => {
  try {
    const db2 = getDatabase();
    const repos = db2.prepare("SELECT * FROM git_repos ORDER BY createdAt DESC").all();
    if (options.json) {
      console.log(JSON.stringify(repos, null, 2));
      return;
    }
    if (repos.length === 0) {
      console.log(source_default.gray("\u6682\u65E0\u5DF2\u4FDD\u5B58\u7684 Git \u4ED3\u5E93"));
      return;
    }
    console.log(source_default.underline("\nGit \u4ED3\u5E93\u5217\u8868:\n"));
    repos.forEach((r2) => {
      console.log(source_default.cyan(`  ${r2.name}`) + source_default.gray(` (${r2.path})`));
      if (r2.remote) console.log(source_default.gray(`    Remote: ${r2.remote}`));
      if (r2.branch) console.log(source_default.gray(`    Branch: ${r2.branch}`));
      console.log("");
    });
  } catch (error) {
    console.error(source_default.red(`\u83B7\u53D6 Git \u4ED3\u5E93\u5217\u8868\u5931\u8D25: ${error.message}`));
  }
});
gitCmd.command("add").description("\u6DFB\u52A0 Git \u4ED3\u5E93").argument("<name>", "\u4ED3\u5E93\u540D\u79F0").argument("<path>", "\u4ED3\u5E93\u8DEF\u5F84").option("-r, --remote <url>", "\u8FDC\u7A0B\u4ED3\u5E93\u5730\u5740").option("-b, --branch <branch>", "\u5206\u652F\u540D", "main").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((name, dirPath, options) => {
  try {
    const db2 = getDatabase();
    const now = (/* @__PURE__ */ new Date()).toISOString();
    const id = `git_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`;
    db2.prepare(
      "INSERT INTO git_repos (id, name, path, remote, branch, lastOpened, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    ).run(id, name, dirPath, options.remote || null, options.branch, now, now, now);
    if (options.json) {
      console.log(JSON.stringify({ success: true, id, name, path: dirPath }, null, 2));
    } else {
      console.log(source_default.green(`\u2713 \u4ED3\u5E93\u5DF2\u6DFB\u52A0: ${name} (${dirPath})`));
    }
  } catch (error) {
    console.error(source_default.red(`\u6DFB\u52A0\u4ED3\u5E93\u5931\u8D25: ${error.message}`));
  }
});
gitCmd.command("remove").alias("rm").description("\u5220\u9664\u4ED3\u5E93").argument("<id>", "\u4ED3\u5E93ID\u6216\u540D\u79F0").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((id, options) => {
  try {
    const db2 = getDatabase();
    let repo = db2.prepare("SELECT * FROM git_repos WHERE id = ?").get(id);
    if (!repo) repo = db2.prepare("SELECT * FROM git_repos WHERE name = ?").get(id);
    if (!repo) {
      console.log(source_default.red(`\u4ED3\u5E93\u4E0D\u5B58\u5728: ${id}`));
      return;
    }
    db2.prepare("DELETE FROM git_repos WHERE id = ?").run(repo.id);
    if (options.json) {
      console.log(JSON.stringify({ success: true, id: repo.id, name: repo.name }, null, 2));
    } else {
      console.log(source_default.green(`\u2713 \u4ED3\u5E93\u5DF2\u5220\u9664: ${repo.name}`));
    }
  } catch (error) {
    console.error(source_default.red(`\u5220\u9664\u4ED3\u5E93\u5931\u8D25: ${error.message}`));
  }
});
gitCmd.command("show").description("\u4ED3\u5E93\u8BE6\u60C5").argument("<id>", "\u4ED3\u5E93ID\u6216\u540D\u79F0").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((id, options) => {
  try {
    const db2 = getDatabase();
    let repo = db2.prepare("SELECT * FROM git_repos WHERE id = ?").get(id);
    if (!repo) repo = db2.prepare("SELECT * FROM git_repos WHERE name = ?").get(id);
    if (!repo) {
      console.log(source_default.red(`\u4ED3\u5E93\u4E0D\u5B58\u5728: ${id}`));
      return;
    }
    if (options.json) {
      console.log(JSON.stringify(repo, null, 2));
      return;
    }
    console.log(source_default.bold(`
\u4ED3\u5E93: ${repo.name}`));
    console.log("-".repeat(40));
    console.log(`${source_default.bold("ID:")} ${repo.id}`);
    console.log(`${source_default.bold("\u8DEF\u5F84:")} ${repo.path}`);
    if (repo.remote) console.log(`${source_default.bold("Remote:")} ${repo.remote}`);
    if (repo.branch) console.log(`${source_default.bold("\u5206\u652F:")} ${repo.branch}`);
    console.log(`${source_default.bold("\u521B\u5EFA\u65F6\u95F4:")} ${repo.createdAt}`);
    if (repo.lastOpened) console.log(`${source_default.bold("\u6700\u540E\u6253\u5F00:")} ${repo.lastOpened}`);
  } catch (error) {
    console.error(source_default.red(`\u83B7\u53D6\u4ED3\u5E93\u8BE6\u60C5\u5931\u8D25: ${error.message}`));
  }
});
gitCmd.command("status").description("\u67E5\u770B\u4ED3\u5E93 Git \u72B6\u6001").argument("<path>", "\u4ED3\u5E93\u8DEF\u5F84").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((dirPath, options) => {
  try {
    if (!fs.existsSync(dirPath)) {
      console.log(source_default.red(`\u8DEF\u5F84\u4E0D\u5B58\u5728: ${dirPath}`));
      return;
    }
    const output = execSync("git status -sb", { cwd: dirPath, encoding: "utf-8" }).trim();
    const porcelainOutput = execSync("git status --porcelain", { cwd: dirPath, encoding: "utf-8" }).trim();
    if (options.json) {
      const lines2 = output.split("\n");
      const branchLine2 = lines2[0] || "";
      const cleanStatus = porcelainOutput === "";
      console.log(JSON.stringify({
        path: dirPath,
        branch: branchLine2,
        clean: cleanStatus,
        changes: porcelainOutput ? porcelainOutput.split("\n").length : 0,
        raw: output
      }, null, 2));
      return;
    }
    console.log(source_default.underline(`
Git \u72B6\u6001: ${dirPath}
`));
    const lines = output.split("\n");
    const branchLine = lines[0] || "";
    console.log(source_default.bold(`  \u5206\u652F: ${branchLine}`));
    if (porcelainOutput) {
      const changeCount = porcelainOutput.split("\n").length;
      console.log(source_default.yellow(`  \u53D8\u66F4: ${changeCount} \u4E2A\u6587\u4EF6`));
      const changeLines = porcelainOutput.split("\n").slice(0, 10);
      changeLines.forEach((line) => {
        const status = line.substring(0, 2);
        const file = line.substring(3);
        const statusColor = status.includes("M") ? source_default.yellow : status.includes("A") ? source_default.green : status.includes("D") ? source_default.red : source_default.gray;
        console.log(`    ${statusColor(status)} ${file}`);
      });
      if (porcelainOutput.split("\n").length > 10) {
        console.log(source_default.gray(`    ... \u8FD8\u6709 ${porcelainOutput.split("\n").length - 10} \u4E2A\u53D8\u66F4`));
      }
    } else {
      console.log(source_default.green("  \u5DE5\u4F5C\u533A\u5E72\u51C0"));
    }
  } catch (error) {
    if (error.status === 128) {
      console.error(source_default.red(`${dirPath} \u4E0D\u662F\u4E00\u4E2A Git \u4ED3\u5E93`));
    } else {
      console.error(source_default.red(`\u83B7\u53D6 Git \u72B6\u6001\u5931\u8D25: ${error.message}`));
    }
  }
});
var openvpnCmd = program.command("openvpn").alias("ovpn").description("OpenVPN \u914D\u7F6E\u7BA1\u7406");
openvpnCmd.command("list").alias("ls").description("\u5217\u51FA OpenVPN \u914D\u7F6E").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((options) => {
  try {
    const db2 = getDatabase();
    const configs = db2.prepare("SELECT id, name, filePath, createdAt FROM openvpn_configs ORDER BY createdAt DESC").all();
    if (options.json) {
      console.log(JSON.stringify(configs, null, 2));
      return;
    }
    if (configs.length === 0) {
      console.log(source_default.gray("\u6682\u65E0 OpenVPN \u914D\u7F6E"));
      return;
    }
    console.log(source_default.underline("\nOpenVPN \u914D\u7F6E\u5217\u8868:\n"));
    configs.forEach((c3) => {
      console.log(source_default.cyan(`  ${c3.name}`) + source_default.gray(` \u2192 ${c3.filePath}`));
      console.log(source_default.gray(`    \u521B\u5EFA: ${c3.createdAt}`));
      console.log("");
    });
  } catch (error) {
    console.error(source_default.red(`\u83B7\u53D6 OpenVPN \u914D\u7F6E\u5217\u8868\u5931\u8D25: ${error.message}`));
  }
});
openvpnCmd.command("add").description("\u6DFB\u52A0 OpenVPN \u914D\u7F6E").argument("<name>", "\u914D\u7F6E\u540D\u79F0").argument("<file_path>", "\u914D\u7F6E\u6587\u4EF6\u8DEF\u5F84").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((name, filePath, options) => {
  try {
    const db2 = getDatabase();
    const now = (/* @__PURE__ */ new Date()).toISOString();
    const id = `ovpn_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`;
    let content = "";
    if (fs.existsSync(filePath)) {
      content = fs.readFileSync(filePath, "utf-8");
    }
    db2.prepare(
      "INSERT INTO openvpn_configs (id, name, filePath, content, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?)"
    ).run(id, name, filePath, content, now, now);
    if (options.json) {
      console.log(JSON.stringify({ success: true, id, name, filePath }, null, 2));
    } else {
      console.log(source_default.green(`\u2713 \u914D\u7F6E\u5DF2\u6DFB\u52A0: ${name} (${filePath})`));
    }
  } catch (error) {
    console.error(source_default.red(`\u6DFB\u52A0\u914D\u7F6E\u5931\u8D25: ${error.message}`));
  }
});
openvpnCmd.command("show").description("\u914D\u7F6E\u8BE6\u60C5").argument("<id>", "\u914D\u7F6EID\u6216\u540D\u79F0").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").option("--no-content", "\u4E0D\u663E\u793A\u914D\u7F6E\u5185\u5BB9").action((id, options) => {
  try {
    const db2 = getDatabase();
    let config = db2.prepare("SELECT * FROM openvpn_configs WHERE id = ?").get(id);
    if (!config) config = db2.prepare("SELECT * FROM openvpn_configs WHERE name = ?").get(id);
    if (!config) {
      console.log(source_default.red(`\u914D\u7F6E\u4E0D\u5B58\u5728: ${id}`));
      return;
    }
    if (options.json) {
      const out = { ...config };
      if (options.content === false) delete out.content;
      console.log(JSON.stringify(out, null, 2));
      return;
    }
    console.log(source_default.bold(`
OpenVPN \u914D\u7F6E: ${config.name}`));
    console.log("-".repeat(40));
    console.log(`${source_default.bold("ID:")} ${config.id}`);
    console.log(`${source_default.bold("\u6587\u4EF6:")} ${config.filePath}`);
    console.log(`${source_default.bold("\u521B\u5EFA\u65F6\u95F4:")} ${config.createdAt}`);
    if (options.content !== false && config.content) {
      console.log(`
${source_default.bold("\u914D\u7F6E\u5185\u5BB9:")}`);
      console.log(source_default.gray(config.content.substring(0, 500) + (config.content.length > 500 ? "..." : "")));
    }
  } catch (error) {
    console.error(source_default.red(`\u83B7\u53D6\u914D\u7F6E\u8BE6\u60C5\u5931\u8D25: ${error.message}`));
  }
});
openvpnCmd.command("delete").alias("del").description("\u5220\u9664\u914D\u7F6E").argument("<id>", "\u914D\u7F6EID\u6216\u540D\u79F0").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((id, options) => {
  try {
    const db2 = getDatabase();
    let config = db2.prepare("SELECT * FROM openvpn_configs WHERE id = ?").get(id);
    if (!config) config = db2.prepare("SELECT * FROM openvpn_configs WHERE name = ?").get(id);
    if (!config) {
      console.log(source_default.red(`\u914D\u7F6E\u4E0D\u5B58\u5728: ${id}`));
      return;
    }
    db2.prepare("DELETE FROM openvpn_configs WHERE id = ?").run(config.id);
    if (options.json) {
      console.log(JSON.stringify({ success: true, id: config.id, name: config.name }, null, 2));
    } else {
      console.log(source_default.green(`\u2713 \u914D\u7F6E\u5DF2\u5220\u9664: ${config.name}`));
    }
  } catch (error) {
    console.error(source_default.red(`\u5220\u9664\u914D\u7F6E\u5931\u8D25: ${error.message}`));
  }
});
openvpnCmd.command("import").description("\u4ECE .ovpn \u6587\u4EF6\u5BFC\u5165").argument("<file_path>", "\u914D\u7F6E\u6587\u4EF6\u8DEF\u5F84").option("-n, --name <name>", "\u914D\u7F6E\u540D\u79F0 (\u9ED8\u8BA4\u4F7F\u7528\u6587\u4EF6\u540D)").option("-j, --json", "JSON\u683C\u5F0F\u8F93\u51FA").action((filePath, options) => {
  try {
    if (!fs.existsSync(filePath)) {
      console.log(source_default.red(`\u6587\u4EF6\u4E0D\u5B58\u5728: ${filePath}`));
      return;
    }
    const content = fs.readFileSync(filePath, "utf-8");
    const name = options.name || path.basename(filePath, ".ovpn") || path.basename(filePath);
    const db2 = getDatabase();
    const now = (/* @__PURE__ */ new Date()).toISOString();
    const id = `ovpn_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`;
    db2.prepare(
      "INSERT INTO openvpn_configs (id, name, filePath, content, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?)"
    ).run(id, name, filePath, content, now, now);
    if (options.json) {
      console.log(JSON.stringify({ success: true, id, name, filePath }, null, 2));
    } else {
      console.log(source_default.green(`\u2713 \u914D\u7F6E\u5DF2\u5BFC\u5165: ${name} (${filePath})`));
    }
  } catch (error) {
    console.error(source_default.red(`\u5BFC\u5165\u914D\u7F6E\u5931\u8D25: ${error.message}`));
  }
});
program.parse();
