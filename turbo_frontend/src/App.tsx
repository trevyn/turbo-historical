import React, { useState, useEffect, FC, FormEvent, SyntheticEvent } from "react";
import { $enum } from "ts-enum-util";
import { Icons } from "./icons";
// import useSWR, { cache as swrCache } from "swr";
import prettyBytes from "pretty-bytes";
import { DateTime } from "luxon";
// import { dequal } from "dequal";
import { Routes, Route, Link, useMatch, useLocation, useNavigate } from "react-router-dom";
import { ApolloClient, InMemoryCache, ApolloProvider } from "@apollo/client";
import * as codegen from "./graphql-codegen";
import ReactPlayer from "react-player";

// if we're at port 3000, assume local dev server, and redirect graphql to port 3020
const port = window.location.port === "3000" ? "3020" : window.location.port;

const apollo = new ApolloClient({
 uri: `${window.location.protocol}//${window.location.hostname}:${port}/graphql`,
 cache: new InMemoryCache(),
 headers: {
  authorization: localStorage.getItem("authorization") || "",
 },
});

const ConstructionComponent: FC = () => (
 <div className="pt-7">
  <div className="relative pt-9/16">
   <ReactPlayer
    className="absolute top-0 left-0"
    width="100%"
    height="100%"
    playing={true}
    controls={true}
    light={false}
    url="http://localhost:3020/filedl/archive2/weird/halu.mp4"
   />
  </div>
 </div>
);

interface NavItem {
 linkpath: string;
 matchpath: string;
 title: string;
 icon: Function;
 component: FC;
}

enum NavItemEnum {
 ToDoList = "ToDoList",
 Bookmarks = "Bookmarks",
 WebSearch = "WebSearch",
 MediaSearch = "MediaSearch",
 Files = "Files",
 ActivityMonitor = "ActivityMonitor",
}

type NavItemTypes =
 | NavItemEnum.ToDoList
 | NavItemEnum.Bookmarks
 | NavItemEnum.WebSearch
 | NavItemEnum.MediaSearch
 | NavItemEnum.Files
 | NavItemEnum.ActivityMonitor;

const navItems: Record<NavItemTypes, NavItem> = {
 ToDoList: {
  linkpath: "/todo",
  matchpath: "/todo",
  title: "To Do List",
  icon: Icons.ClipboardList,
  component: ConstructionComponent,
 },
 Bookmarks: {
  linkpath: "/bookmarks",
  matchpath: "/bookmarks",
  title: "Bookmarks",
  icon: Icons.Bookmark,
  component: ConstructionComponent,
 },
 WebSearch: {
  linkpath: "/websearch",
  matchpath: "/websearch/*",
  title: "Web Search",
  icon: Icons.Search,
  component: ConstructionComponent,
 },
 MediaSearch: {
  linkpath: "/mediasearch",
  matchpath: "/mediasearch",
  title: "Media Search",
  icon: Icons.Film,
  component: ConstructionComponent,
 },
 Files: {
  linkpath: "/files",
  matchpath: "/files/*",
  title: "Files",
  icon: Icons.Folder,
  component: ConstructionComponent,
 },
 ActivityMonitor: {
  linkpath: "/activitymonitor",
  matchpath: "/activitymonitor",
  title: "Activity Monitor",
  icon: Icons.Globe,
  component: ConstructionComponent,
 },
};

const navItemsEnum = Object.entries(Object.keys(navItems)).reduce(
 (obj, [key, value]) => ({ ...obj, [value]: key }),
 {}
);

// let JUST_FETCHED_KEY: string | null = null;

// $enum(navItemsEnum).forEach((value, key, wrappedEnum, index) => {
//  console.log("foo", value, key, wrappedEnum, index);
// });

// $enum.visitValue("ActivityMonitor").with({
//  [NavItemEnum.ToDoList]: () => <ConstructionComponent />,
//  [NavItemEnum.ActivityMonitor]: () => <ConstructionComponent />,
// });

function App() {
 // let [navbarState, setNavbarState] = useState(NavbarItems.Files);
 // let [searchResult, setSearchResult] = useState([] as SearchItem[]);
 // let [filesPath, setFilesPath] = useState("");
 let isMatch = useMatch;

 return (
  <ApolloProvider client={apollo}>
   <div className="h-screen flex bg-white">
    <div className="fixed h-screen flex flex-shrink-0">
     <div className="flex flex-col w-64 border-r border-gray-200 pb-4 bg-gray-100">
      {/* Sidebar component */}
      <div className="h-0 flex-1 flex flex-col overflow-y-auto">
       {/* Sidebar Search */}
       <div className="px-3 mt-9">
        <label htmlFor="search" className="sr-only">
         Search
        </label>
        <div className="mt-1 relative rounded-md shadow-sm">
         <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
          <Icons.SearchSmall className="mr-3 h-4 w-4 text-gray-400" />
         </div>
         <input
          id="search"
          autoComplete="off"
          className="form-input block w-full pl-9 text-sm leading-5 focus:border-indigo-300 focus:shadow-outline-indigo"
          placeholder={'Search (Press "/" to focus)'}
         />
        </div>
       </div>
       {/* Navigation */}
       <nav className="px-3 mt-9">
        <div className="space-y-1">
         {Object.values(navItems).map((item: NavItem) => (
          <Link to={item.linkpath}>
           <div
            className={`${
             isMatch(item.matchpath)
              ? "hover:bg-gray-300 bg-gray-200 text-gray-900"
              : "hover:bg-gray-200 focus:bg-gray-200 text-gray-500 hover:text-gray-900"
            } group flex items-center px-2 py-2 text-sm leading-5 font-medium rounded-md focus:outline-none`}
           >
            {item.icon({
             className: `${
              isMatch(item.matchpath)
               ? "group-hover:text-gray-600 text-gray-500"
               : "group-hover:text-gray-500 text-gray-400"
             } mr-3 h-6 w-6 group-focus:text-gray-600 transition ease-in-out duration-150`,
            })}
            {item.title}
           </div>
          </Link>
         ))}
        </div>
        <div className="mt-8">
         {/* Secondary navigation */}
         <h3
          className="cursor-default px-3 text-xs leading-4 font-semibold text-gray-500 uppercase tracking-wider"
          id="dock-headline"
         >
          Dock {/*(Drag things here!)*/}
         </h3>
         <div className="mt-1 space-y-1" role="group" aria-labelledby="dock-headline">
          <a
           href="/"
           className="group flex items-center px-3 py-2 text-sm leading-5 font-medium text-gray-500 rounded-md hover:text-gray-900 hover:bg-gray-50 focus:outline-none focus:bg-gray-50"
          >
           <span className="w-2.5 h-2.5 mr-4 bg-indigo-500 rounded-full"></span>
           <span className="truncate">Chill Institute</span>
          </a>

          <a
           href="/"
           className="group flex items-center px-3 py-2 text-sm leading-5 font-medium text-gray-500 rounded-md hover:text-gray-900 hover:bg-gray-50 focus:outline-none focus:bg-gray-50"
          >
           <span className="w-2.5 h-2.5 mr-4 bg-teal-400 rounded-full"></span>
           <span className="truncate">ShowRSS</span>
          </a>

          <a
           href="/"
           className="group flex items-center px-3 py-2 text-sm leading-5 font-medium text-gray-500 rounded-md hover:text-gray-900 hover:bg-gray-50 focus:outline-none focus:bg-gray-50"
          >
           <span className="w-2.5 h-2.5 mr-4 bg-orange-500 rounded-full"></span>
           <span className="truncate">put.io</span>
          </a>
          <a
           href="/"
           className="group flex items-center px-3 py-2 text-sm leading-5 font-medium text-gray-500 rounded-md hover:text-gray-900 hover:bg-gray-50 focus:outline-none focus:bg-gray-50"
          >
           <span className="w-2.5 h-2.5 mr-4 bg-pink-500 rounded-full"></span>
           <span className="truncate">gdrive</span>
          </a>
         </div>
        </div>
       </nav>
      </div>
     </div>
    </div>
    <div className="flex flex-col ml-64 w-0 flex-1">
     <div className="bg-gray-50 h-screen pt-4 px-3 sm:px-8">
      <Routes>
       {/* {Object.values(navItems).map((item: NavItem) => (
        <Route path={item.path} element={<div>I AM THE {item.title} PAGE</div>} />
       ))} */}

       <Route
        path="/setpassword/*"
        element={
         <nav className="pl-5 p-8 pb-10 mt-px flex items-center text-sm leading-5 font-medium">
          <SetPasswordComponent />
         </nav>
        }
       />

       {$enum(navItemsEnum)
        .getKeys()
        .map((item: NavItemTypes) =>
         $enum.mapValue(item).with({
          [NavItemEnum.ActivityMonitor]: (
           <Route
            path={navItems[item].matchpath}
            element={
             <nav className="pl-5 p-8 pb-10 mt-px flex items-center text-sm leading-5 font-medium">
              <ActivityMonitorComponent />
             </nav>
            }
           />
          ),

          [NavItemEnum.ToDoList]: (
           <Route
            path={navItems[item].matchpath}
            element={
             // <nav className="pl-5 p-8 pb-10 mt-px flex items-center text-sm leading-5 font-medium">
             //  <span className="cursor-default text-gray-500">
             <ConstructionComponent />
             //  </span>
             // </nav>
            }
           />
          ),

          [NavItemEnum.Bookmarks]: (
           <Route
            path={navItems[item].matchpath}
            element={
             <>
              <nav className="pl-5 p-8 pb-10 mt-px flex items-center text-sm leading-5 font-medium">
               <span className="cursor-default text-gray-500">{navItems[item].title}</span>
              </nav>
              <BookmarksComponent />
             </>
            }
           />
          ),

          [NavItemEnum.WebSearch]: (
           <Route
            path={navItems[item].matchpath}
            element={
             // <SetSearchResult.Provider value={setSearchResult}>
             <SearchComponent />
             // </SetSearchResult.Provider>
            }
           />
          ),

          [NavItemEnum.MediaSearch]: (
           <Route
            path={navItems[item].matchpath}
            element={
             // <SetSearchResult.Provider value={setSearchResult}>
             <ConstructionComponent />
             // </SetSearchResult.Provider>
            }
           />
          ),

          [NavItemEnum.Files]: (
           <Route path={navItems[item].matchpath} element={<FilesComponent />} />
          ),
         })
        )}
      </Routes>
     </div>
    </div>
   </div>
  </ApolloProvider>
 );
}

const BookmarksComponent: FC = () => {
 const { data } = codegen.useGetBookmarksQuery();
 return (data && data.getBookmarks && <ResultsComponent results={data.getBookmarks} />) ?? <></>;
};

const ActivityMonitorComponent: FC = () => {
 const { data } = codegen.useGetActivityMonitorQuery();
 return <div>{data && data.getActivityMonitor && JSON.stringify(data.getActivityMonitor)}</div>;
};

const SetPasswordComponent: FC = () => {
 let { pathname } = useLocation();
 let password = pathname.slice("/setpassword/".length);
 localStorage.setItem("authorization", `Bearer ${password}`);

 return <>Password set! Reload page pls.</>;
};

const SearchComponent: FC = () => {
 // const setSearchResult = useContext(SetSearchResult);
 let { pathname } = useLocation();
 let path = pathname.slice("/websearch/".length);
 let [value, setValue] = useState(decodeURI(path));
 let noResults: any = null;
 let [searchResults, setSearchResults] = useState(noResults);

 console.log("render");

 const [getSearch, { loading: searchLoading, data: searchData }] = codegen.useSearchLazyQuery({
  fetchPolicy: "no-cache", // = network-only + do not add to cache
  nextFetchPolicy: "cache-only",
  onCompleted: (props: codegen.SearchQuery) => {
   console.log("onCompleted", props);
   setSearchResults(props.search);
  },
 });

 let navigate = useNavigate();

 useEffect(() => {
  console.log("useEffect");
  if (path) {
   getSearch({ variables: { query: decodeURI(path), forceScrape: true } });
   // getSearch({ variables: { query: path, forceScrape: true } });
  }
  // eslint-disable-next-line react-hooks/exhaustive-deps
 }, []);

 return (
  <>
   <div className="m-6 mb-8 flex-1 flex items-center justify-center px-2">
    <div className="max-w-xl w-full">
     <label htmlFor="search" className="sr-only">
      Search
     </label>
     <div className="relative">
      <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
       <svg className="h-5 w-5 text-gray-400" fill="currentColor" viewBox="0 0 20 20">
        <path
         fillRule="evenodd"
         d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z"
         clipRule="evenodd"
        />
       </svg>
      </div>
      <input
       autoFocus
       autoComplete="off"
       id="search"
       value={value}
       onKeyUp={async (e: React.KeyboardEvent<HTMLInputElement> & { target: HTMLInputElement }) => {
        if (e.keyCode === 13) {
         navigate("/websearch/" + e.target.value);
         setValue(e.target.value);
         getSearch({ variables: { query: e.target.value, forceScrape: true } });
        }
       }}
       onChange={async (e: FormEvent<HTMLInputElement> & { target: HTMLInputElement }) => {
        navigate("/websearch/" + e.target.value);
        setValue(e.target.value);
        getSearch({ variables: { query: e.target.value, forceScrape: false } });
       }}
       className="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:border-indigo-300 focus:shadow-outline-indigo text-sm transition duration-150 ease-in-out"
       placeholder="Search"
       type="search"
      />
     </div>
    </div>
   </div>
   <div className="w-full pb-8 h-8 flex items-center justify-center">
    <div className="text-4xl italic font-light text-gray-200">
     {searchData === undefined && searchLoading === true ? "searching..." : ""}
    </div>
   </div>
   <ResultsComponent results={searchResults} />

   {/* <div>{JSON.stringify(data)}</div> */}
  </>
 );
};

const ResultsComponent: FC<{
 results: (codegen.SearchQueryResultItem | codegen.BookmarkQueryResultItem)[];
}> = ({ results }) => {
 const [setHostAffectionMutation] = codegen.useSetHostAffectionMutation();
 const [setBookmarkedMutation] = codegen.useSetBookmarkedMutation();

 // console.log(results);

 return (
  <>
   {results &&
    results.map(item => (
     <div
      className="group flex cursor-pointer p-3 pt-2 hover:bg-indigo-100 rounded-lg"
      onClick={(e: SyntheticEvent) => {
       console.log("outer");

       if (item.url) {
        const anchor = document.createElement("a");

        Object.assign(anchor, {
         target: "_blank",
         href: item.url,
         rel: "noopener noreferrer",
        }).click();

        // window.open(item.url);
        e.stopPropagation();
       }
      }}
     >
      <div className="flex-none w-12 h-24 mr-2">
       <Icon
        svgFunction={Icons.EmojiHappy}
        className={item.hostaffection > 0 ? "animate-ping2" : "animate-ping3"}
        selected={item.hostaffection > 0}
        onClick={(e: SyntheticEvent) => {
         if (item.host)
          setHostAffectionMutation({
           variables: { host: item.host, affection: item.hostaffection === 1 ? 0 : 1 },
          });
         // let newSearchResult = searchResult.slice();
         // newSearchResult[itemidx] = {
         //  ...item,
         //  thumbedUp: !item.thumbedUp,
         //  thumbedDown: false,
         // };
         // setSearchResult(newSearchResult);
         e.stopPropagation();
        }}
       />
       <Icon
        svgFunction={Icons.Bookmark}
        className={
         (item.bookmarked ? "animate-ping2" : "animate-ping3") +
         " " +
         (!(item.hostaffection > 0) && item.bookmarked
          ? "-translate-y-8 group-hover:translate-y-0"
          : "")
        }
        selected={item.bookmarked}
        onClick={(e: SyntheticEvent) => {
         if (item.url)
          setBookmarkedMutation({
           variables: {
            url: item.url,
            bookmarked: !item.bookmarked,
           },
          });
         // let newSearchResult = searchResult.slice();
         // newSearchResult[itemidx] = { ...item, bookmarked: !item.bookmarked };
         // apiSetBookmark(newSearchResult[itemidx].origurl || newSearchResult[itemidx].url);
         // setSearchResult(newSearchResult);
         e.stopPropagation();
        }}
       />
       <Icon
        svgFunction={Icons.ThumbDown}
        className={
         (item.hostaffection < 0 ? "animate-ping2" : "animate-ping3") +
         " " +
         (item.hostaffection < 0
          ? item.bookmarked
            ? "-translate-y-8 group-hover:translate-y-0"
            : "-translate-y-16 group-hover:translate-y-0"
          : "")
        }
        selected={item.hostaffection < 0}
        onClick={(e: SyntheticEvent) => {
         if (item.host)
          setHostAffectionMutation({
           variables: {
            host: item.host,
            affection: item.hostaffection === -1 ? 0 : -1,
           },
          });

         // let newSearchResult = searchResult.slice();
         // newSearchResult[itemidx] = {
         //  ...item,
         //  thumbedDown: !item.thumbedDown,
         //  thumbedUp: false,
         // };
         // setSearchResult(newSearchResult);
         e.stopPropagation();
        }}
       />
      </div>
      <div>
       <div
        className="block pt-0.5 text-gray-600 group-hover:underline text-lg font-medium"
        dangerouslySetInnerHTML={{ __html: item.title }}
        // rel="noopener noreferrer"
        // target="_blank"
        // onClick={async () => {
        //  const result = await fetch(
        //   `http://localhost:3030/monolith/${encodeURIComponent(item.url)}`
        //  );
        //  console.log(DOMPurify.sanitize(await result.text()));
        // }}
        // href={item.url}
       ></div>
       <div>
        <span
         className="text-sm text-green-500"
         dangerouslySetInnerHTML={{
          __html: "searchHighlightedUrl" in item ? item.searchHighlightedUrl : item.url,
         }}
        ></span>
        <span
         className="pl-4 text-sm text-gray-500 hover:text-gray-700 underline"
         onClick={(e: SyntheticEvent) => {
          console.log("inner");
          if (item.url) {
           const anchor = document.createElement("a");

           Object.assign(anchor, {
            target: "_blank",
            href: `https://archive.vn/?run=1&url=${item.url}`,
            rel: "noopener noreferrer",
           }).click();

           // window.open(`https://archive.vn/?run=1&url=${item.url}`);
           e.stopPropagation();
          }
         }}
        >
         archive
        </span>
       </div>
       <div
        className="text-sm text-gray-500"
        dangerouslySetInnerHTML={{ __html: item.snippet }}
       ></div>
       <div className="text-sm text-gray-300">
        {"rank" in item ? `Rank ${item.rank.toFixed(2)}` : ""}
       </div>
      </div>
     </div>
    ))}
   <div className="w-full pb-8 h-64 flex items-center justify-center">
    <div className="text-4xl italic font-light text-gray-200">fin</div>
   </div>
  </>
 );
};

// const NavBar: FC = ({ children }) => {
//  return (
//   <nav className="z-10 bg-white shadow-md fixed w-full select-none">
//    <div className="max-w-7xl mx-auto px-2 sm:px-6 lg:px-8">
//     <div className="relative flex justify-between h-16">
//      <div className="flex-1 flex items-stretch justify-center sm:justify-start">
//       <div className="sm:ml-2 flex">{children}</div>
//      </div>
//     </div>
//    </div>
//   </nav>
//  );
// };

// const NavItem: FC<{ active: boolean; onClick: Function }> = ({ active, onClick, children }) => {
//  return (
//   <div
//    onClick={() => onClick()}
//    className={`cursor-pointer mx-3 inline-flex items-center px-2 pt-1 border-b-2 text-sm font-medium leading-5 focus:outline-none transition duration-150 ease-in-out ${
//     active
//      ? "border-indigo-500 text-gray-900 focus:border-indigo-700"
//      : "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 focus:text-gray-700 focus:border-gray-300"
//    }`}
//   >
//    {children}
//   </div>
//  );
// };

// const SetSearchResult = React.createContext<React.Dispatch<Array<codegen.ResultItem>> | null>(null);

// enum NavbarItems {
//  ToDoList = "To Do List",
//  Bookmarks = "Bookmarks",
//  Search = "Search",
//  Files = "Files",
//  Settings = "Settings",
// }

// const fetcher = async (url: any) => fetch(url).then(r => r.json());

const FilesComponent: FC = () => {
 let { pathname } = useLocation();
 let path = pathname.slice("/files/".length);

 const { data, error } = codegen.useGetRcloneItemsQuery({
  variables: {
   path: decodeURI(path),
  },
 });

 // let key = "http://localhost:3020/listjson/" + path.split("/").map(encodeURI).join("/");
 // let cached = swrCache.get(key);

 // const { data, error } = useSWR(
 //  "http://localhost:3020/listjson/" + path,
 //  JUST_FETCHED_KEY === key && cached
 //   ? () => {
 //      JUST_FETCHED_KEY = null;
 //      return cached;
 //     }
 //   : fetcher
 // );

 if (error) return <div>failed to load</div>;
 if (!data || !data.getRcloneItems) return <div>loading...</div>;

 let patharray = path.split("/");

 return (
  <>
   {/* <div className="ml-5 mt-6 md:flex md:items-center md:justify-between">
    <div className="mt-4 flex-shrink-0 flex md:mt-0 md:ml-4">
     <span className="shadow-sm rounded-md">
      <button
       type="button"
       className="inline-flex items-center px-4 py-2 border border-gray-300 text-sm leading-5 font-medium rounded-md text-gray-700 bg-white hover:text-gray-500 focus:outline-none focus:shadow-outline-blue focus:border-blue-300 active:text-gray-800 active:bg-gray-50 transition duration-150 ease-in-out"
      >
       Edit
      </button>
     </span>
     <span className="ml-3 shadow-sm rounded-md">
      <button
       type="button"
       className="inline-flex items-center px-4 py-2 border border-transparent text-sm leading-5 font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-500 focus:outline-none focus:shadow-outline-indigo focus:border-indigo-700 active:bg-indigo-700 transition duration-150 ease-in-out"
      >
       Publish
      </button>
     </span>
    </div>
   </div> */}
   <div>
    {/* <nav className="sm:hidden">
     <a
      href="/"
      className="flex items-center text-sm leading-5 font-medium text-gray-500 hover:text-gray-700 transition duration-150 ease-in-out"
     >
      <svg
       className="flex-shrink-0 -ml-1 mr-1 h-5 w-5 text-gray-400"
       viewBox="0 0 20 20"
       fill="currentColor"
      >
       <path
        fillRule="evenodd"
        d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z"
        clipRule="evenodd"
       />
      </svg>
      Back
     </a>
    </nav> */}
    <nav className="pl-5 p-8 pb-10 mt-px flex items-center text-sm leading-5 font-medium">
     <span className="cursor-pointer text-gray-500 hover:text-gray-700 hover:underline transition duration-150 ease-in-out">
      Files
     </span>
     <Icons.ChevronRightSmall className="flex-shrink-0 mx-2 h-5 w-5 text-gray-400" />
     <Link to="/files">
      <span
       // onClick={() => setPath("")}
       className="cursor-pointer text-gray-500 hover:text-gray-700 hover:underline form-selecttransition duration-150 ease-in-out"
      >
       gdrive
      </span>
     </Link>
     {path === "" ? (
      <></>
     ) : (
      Object.keys(patharray).map(key => {
       let i = parseInt(key);
       let newPath = [];

       for (let j = 0; j <= i; j++) {
        newPath.push(patharray[j]);
       }

       let newPathString = newPath.join("/");

       return i === patharray.length - 1 ? ( // last item
        <>
         <Icons.ChevronRightSmall className="flex-shrink-0 mx-2 h-5 w-5 text-gray-400" />
         <span className="cursor-default text-gray-500">{decodeURI(patharray[i])}</span>
        </>
       ) : (
        <>
         <Icons.ChevronRightSmall className="flex-shrink-0 mx-2 h-5 w-5 text-gray-400" />
         <Link to={"/files/" + newPathString + "/"}>
          <span
           // onClick={() => setPath(newPathString)}
           className="cursor-pointer text-gray-500 hover:text-gray-700 hover:underline form-selecttransition duration-150 ease-in-out"
          >
           {decodeURI(patharray[i])}
          </span>
         </Link>
        </>
       );
      })
     )}
    </nav>
   </div>

   <div className="flex flex-col">
    <div className="-my-2 overflow-x-auto sm:-mx-6 lg:-mx-8">
     <div className="py-2 align-middle inline-block sm:px-6 lg:px-8">
      <div className="shadow overflow-hidden border-b border-t border-gray-200 sm:rounded-lg">
       <table className=" divide-y divide-gray-200">
        <tbody className="bg-white divide-y divide-gray-200">
         {data.getRcloneItems.map((item: codegen.RcloneItemQueryResultItem) => (
          <FileEntry2 item={item} />
         ))}
        </tbody>
       </table>
      </div>
     </div>
    </div>
   </div>
   <div className="pt-96 clearfix"></div>
   <div className="pt-96 clearfix">{JSON.stringify(data)}</div>
  </>
 );
};

const FileEntry2: FC<{ item: codegen.RcloneItemQueryResultItem }> = ({ item }) => {
 let navigate = useNavigate();

 return (
  <tr className="hover:bg-gray-50 cursor-pointer">
   <td>
    {/* <Link to={"/files/" + item.Path}> */}
    <div
     className="px-6 py-4 whitespace-no-wrap flex items-center"
     onClick={async e => {
      console.log("item.Path: " + item.path);
      // let key = "http://localhost:3020/listjson/" + item.Path.split("/").map(encodeURI).join("/");

      // if (!swrCache.get(key)) {
      //  console.log("fetching key: http://localhost:3020/listjson/" + item.Path);
      //  let json = await (await fetch("http://localhost:3020/listjson/" + item.Path)).json();
      //  swrCache.set(key, json);
      //  JUST_FETCHED_KEY = key;
      // }

      if (item.isDir) {
       navigate("/files/" + item.path + "/");
      } else {
       // is file
       const anchor = document.createElement("a");

       Object.assign(anchor, {
        target: "_blank",
        href: "http://localhost:3020/filedl/" + item.path,
        rel: "noopener noreferrer",
       }).click();

       e.stopPropagation();
      }
     }}
    >
     <div className="flex-shrink-0 h-10 w-10">
      <img
       className="h-10 w-10 rounded-full"
       src="https://images.unsplash.com/photo-1494790108377-be9c29b29330?ixlib=rb-1.2.1&amp;ixid=eyJhcHBfaWQiOjEyMDd9&amp;auto=format&amp;fit=facearea&amp;facepad=4&amp;w=256&amp;h=256&amp;q=60"
       alt=""
      />
     </div>
     <div className="ml-4">
      <div className="text-sm leading-5 font-medium text-gray-700">{item.name}</div>
      <div className="text-sm leading-5 text-gray-400 font-thin">
       {item.size && item.size >= 0 && `${prettyBytes(item.size)}, `}
       {item.dirSize && `${prettyBytes(item.dirSize)}, `}
       {item.modTime &&
        `${DateTime.fromISO(item.modTime).toRelative()} (${DateTime.fromISO(item.modTime).toFormat(
         "MMM d, yyyy"
        )})`}
      </div>
     </div>
    </div>
    {/* </Link> */}
   </td>
  </tr>
 );
};

const Icon: FC<{ svgFunction: FC<{ className: string }>; selected: boolean } & Partial<any>> = ({
 className,
 svgFunction,
 selected,
 ...otherProps
}) => {
 const selectedClasses =
  "opacity-100 text-indigo-400 group-hover:text-indigo-500 hover:text-indigo-600";
 const unselectedClasses = "opacity-0 text-gray-400 hover:text-gray-500";

 return svgFunction({
  className:
   className +
   " transform group-hover:opacity-100 py-1 px-3 h-8 w-12 transition-color-transform ease-in-out duration-150 " +
   (selected ? selectedClasses : unselectedClasses),
  ...otherProps,
 });
};

export default App;
