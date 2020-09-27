import React from "react";

export const Icons = {
 ThumbDown: (props: any) => (
  <svg
   {...props}
   xmlns="http://www.w3.org/2000/svg"
   fill="none"
   viewBox="0 0 24 24"
   stroke="currentColor"
  >
   <path
    strokeLinecap="round"
    strokeLinejoin="round"
    strokeWidth={2}
    d="M10 14H5.236a2 2 0 01-1.789-2.894l3.5-7A2 2 0 018.736 3h4.018a2 2 0 01.485.06l3.76.94m-7 10v5a2 2 0 002 2h.096c.5 0 .905-.405.905-.904 0-.715.211-1.413.608-2.008L17 13V4m-7 10h2m5-10h2a2 2 0 012 2v6a2 2 0 01-2 2h-2.5"
   />
  </svg>
 ),

 ThumbUp: (props: any) => (
  <svg
   {...props}
   xmlns="http://www.w3.org/2000/svg"
   fill="none"
   viewBox="0 0 24 24"
   stroke="currentColor"
  >
   <path
    strokeLinecap="round"
    strokeLinejoin="round"
    strokeWidth={2}
    d="M14 10h4.764a2 2 0 011.789 2.894l-3.5 7A2 2 0 0115.263 21h-4.017c-.163 0-.326-.02-.485-.06L7 20m7-10V5a2 2 0 00-2-2h-.095c-.5 0-.905.405-.905.905 0 .714-.211 1.412-.608 2.006L7 11v9m7-10h-2M7 20H5a2 2 0 01-2-2v-6a2 2 0 012-2h2.5"
   />
  </svg>
 ),

 Bookmark: (props: any) => (
  <svg
   {...props}
   xmlns="http://www.w3.org/2000/svg"
   fill="none"
   viewBox="0 0 24 24"
   stroke="currentColor"
  >
   <path
    strokeLinecap="round"
    strokeLinejoin="round"
    strokeWidth={2}
    d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z"
   />
  </svg>
 ),

 X: (props: any) => (
  <svg
   {...props}
   xmlns="http://www.w3.org/2000/svg"
   fill="none"
   viewBox="0 0 24 24"
   stroke="currentColor"
  >
   <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
  </svg>
 ),

 ClipboardList: (props: any) => (
  <svg
   {...props}
   xmlns="http://www.w3.org/2000/svg"
   fill="none"
   viewBox="0 0 24 24"
   stroke="currentColor"
  >
   <path
    strokeLinecap="round"
    strokeLinejoin="round"
    strokeWidth={2}
    d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01"
   />
  </svg>
 ),

 Search: (props: any) => (
  <svg
   {...props}
   xmlns="http://www.w3.org/2000/svg"
   fill="none"
   viewBox="0 0 24 24"
   stroke="currentColor"
  >
   <path
    strokeLinecap="round"
    strokeLinejoin="round"
    strokeWidth={2}
    d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
   />
  </svg>
 ),

 SearchSmall: (props: any) => (
  <svg {...props} xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
   <path
    fillRule="evenodd"
    d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z"
    clipRule="evenodd"
   />
  </svg>
 ),

 Globe: (props: any) => (
  <svg
   {...props}
   xmlns="http://www.w3.org/2000/svg"
   fill="none"
   viewBox="0 0 24 24"
   stroke="currentColor"
  >
   <path
    strokeLinecap="round"
    strokeLinejoin="round"
    strokeWidth={2}
    d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
   />
  </svg>
 ),

 Folder: (props: any) => (
  <svg
   {...props}
   xmlns="http://www.w3.org/2000/svg"
   fill="none"
   viewBox="0 0 24 24"
   stroke="currentColor"
  >
   <path
    strokeLinecap="round"
    strokeLinejoin="round"
    strokeWidth={2}
    d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
   />
  </svg>
 ),

 EmojiHappy: (props: any) => (
  <svg
   {...props}
   xmlns="http://www.w3.org/2000/svg"
   fill="none"
   viewBox="0 0 24 24"
   stroke="currentColor"
  >
   <path
    strokeLinecap="round"
    strokeLinejoin="round"
    strokeWidth={2}
    d="M14.828 14.828a4 4 0 01-5.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
   />
  </svg>
 ),

 ChevronRightSmall: (props: any) => (
  <svg {...props} xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
   <path
    fillRule="evenodd"
    d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z"
    clipRule="evenodd"
   />
  </svg>
 ),

 Play: (props: any) => (
  <svg
   {...props}
   xmlns="http://www.w3.org/2000/svg"
   fill="none"
   viewBox="0 0 24 24"
   stroke="currentColor"
  >
   <path
    strokeLinecap="round"
    strokeLinejoin="round"
    strokeWidth={2}
    d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
   />
   <path
    strokeLinecap="round"
    strokeLinejoin="round"
    strokeWidth={2}
    d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
   />
  </svg>
 ),

 Film: (props: any) => (
  <svg
   {...props}
   xmlns="http://www.w3.org/2000/svg"
   fill="none"
   viewBox="0 0 24 24"
   stroke="currentColor"
  >
   <path
    strokeLinecap="round"
    strokeLinejoin="round"
    strokeWidth={2}
    d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z"
   />
  </svg>
 ),
};
