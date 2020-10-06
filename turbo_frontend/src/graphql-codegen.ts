import { gql } from '@apollo/client';
import * as Apollo from '@apollo/client';
import * as ApolloReactHooks from '@apollo/client';
export type Maybe<T> = T | null;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: string;
  String: string;
  Boolean: boolean;
  Int: number;
  Float: number;
  /** i53: 53-bit signed integer; represented as `i53`/`i64` in Rust, `Float` in GraphQL, `number` in TypeScript. */
  i53: number;
};

export type ActivityMonitor = {
  __typename?: 'ActivityMonitor';
  totalMemory: Scalars['Int'];
  usedMemory: Scalars['Int'];
  availableMemory: Scalars['Int'];
  totalSwap: Scalars['Int'];
  usedSwap: Scalars['Int'];
};

export type RcloneItemQueryResultItem = {
  __typename?: 'RcloneItemQueryResultItem';
  path: Scalars['String'];
  name: Scalars['String'];
  size: Scalars['i53'];
  mimeType: Scalars['String'];
  modTime: Scalars['String'];
  isDir: Scalars['Boolean'];
  dirSize?: Maybe<Scalars['i53']>;
};

export type SearchQueryResultItem = {
  __typename?: 'SearchQueryResultItem';
  searchHighlightedUrl: Scalars['String'];
  title: Scalars['String'];
  snippet: Scalars['String'];
  url: Scalars['String'];
  host: Scalars['String'];
  bookmarked: Scalars['Boolean'];
  hostaffection: Scalars['Int'];
  rank: Scalars['Float'];
};

export type Query = {
  __typename?: 'Query';
  getBookmarks: Array<BookmarkQueryResultItem>;
  search: Array<SearchQueryResultItem>;
  getActivityMonitor: ActivityMonitor;
  getCommitHash: Scalars['String'];
  getRcloneItems: Array<RcloneItemQueryResultItem>;
};


export type QuerySearchArgs = {
  query: Scalars['String'];
  forceScrape: Scalars['Boolean'];
};


export type QueryGetRcloneItemsArgs = {
  path: Scalars['String'];
};

export type Mutations = {
  __typename?: 'Mutations';
  setHostAffection: Scalars['String'];
  setBookmarked: Scalars['String'];
};


export type MutationsSetHostAffectionArgs = {
  host: Scalars['String'];
  affection: Scalars['Int'];
};


export type MutationsSetBookmarkedArgs = {
  url: Scalars['String'];
  bookmarked: Scalars['Boolean'];
};


export type BookmarkQueryResultItem = {
  __typename?: 'BookmarkQueryResultItem';
  bookmarkTimestamp: Scalars['Float'];
  title: Scalars['String'];
  snippet: Scalars['String'];
  url: Scalars['String'];
  host: Scalars['String'];
  bookmarked: Scalars['Boolean'];
  hostaffection: Scalars['Int'];
};

export type GetBookmarksQueryVariables = Exact<{ [key: string]: never; }>;


export type GetBookmarksQuery = (
  { __typename?: 'Query' }
  & { getBookmarks: Array<(
    { __typename?: 'BookmarkQueryResultItem' }
    & Pick<BookmarkQueryResultItem, 'bookmarkTimestamp' | 'title' | 'snippet' | 'url' | 'host' | 'bookmarked' | 'hostaffection'>
  )> }
);

export type SearchQueryVariables = Exact<{
  query: Scalars['String'];
  forceScrape: Scalars['Boolean'];
}>;


export type SearchQuery = (
  { __typename?: 'Query' }
  & { search: Array<(
    { __typename?: 'SearchQueryResultItem' }
    & Pick<SearchQueryResultItem, 'searchHighlightedUrl' | 'title' | 'snippet' | 'url' | 'host' | 'bookmarked' | 'hostaffection' | 'rank'>
  )> }
);

export type GetActivityMonitorQueryVariables = Exact<{ [key: string]: never; }>;


export type GetActivityMonitorQuery = (
  { __typename?: 'Query' }
  & { getActivityMonitor: (
    { __typename?: 'ActivityMonitor' }
    & Pick<ActivityMonitor, 'totalMemory' | 'usedMemory' | 'availableMemory' | 'totalSwap' | 'usedSwap'>
  ) }
);

export type GetCommitHashQueryVariables = Exact<{ [key: string]: never; }>;


export type GetCommitHashQuery = (
  { __typename?: 'Query' }
  & Pick<Query, 'getCommitHash'>
);

export type GetRcloneItemsQueryVariables = Exact<{
  path: Scalars['String'];
}>;


export type GetRcloneItemsQuery = (
  { __typename?: 'Query' }
  & { getRcloneItems: Array<(
    { __typename?: 'RcloneItemQueryResultItem' }
    & Pick<RcloneItemQueryResultItem, 'path' | 'name' | 'size' | 'mimeType' | 'modTime' | 'isDir' | 'dirSize'>
  )> }
);

export type SetHostAffectionMutationVariables = Exact<{
  host: Scalars['String'];
  affection: Scalars['Int'];
}>;


export type SetHostAffectionMutation = (
  { __typename?: 'Mutations' }
  & Pick<Mutations, 'setHostAffection'>
);

export type SetBookmarkedMutationVariables = Exact<{
  url: Scalars['String'];
  bookmarked: Scalars['Boolean'];
}>;


export type SetBookmarkedMutation = (
  { __typename?: 'Mutations' }
  & Pick<Mutations, 'setBookmarked'>
);


export const GetBookmarksDocument = gql`
    query getBookmarks {
  getBookmarks {
    bookmarkTimestamp
    title
    snippet
    url
    host
    bookmarked
    hostaffection
  }
}
    `;

/**
 * __useGetBookmarksQuery__
 *
 * To run a query within a React component, call `useGetBookmarksQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetBookmarksQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetBookmarksQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetBookmarksQuery(baseOptions?: ApolloReactHooks.QueryHookOptions<GetBookmarksQuery, GetBookmarksQueryVariables>) {
        return ApolloReactHooks.useQuery<GetBookmarksQuery, GetBookmarksQueryVariables>(GetBookmarksDocument, baseOptions);
      }
export function useGetBookmarksLazyQuery(baseOptions?: ApolloReactHooks.LazyQueryHookOptions<GetBookmarksQuery, GetBookmarksQueryVariables>) {
          return ApolloReactHooks.useLazyQuery<GetBookmarksQuery, GetBookmarksQueryVariables>(GetBookmarksDocument, baseOptions);
        }
export type GetBookmarksQueryHookResult = ReturnType<typeof useGetBookmarksQuery>;
export type GetBookmarksLazyQueryHookResult = ReturnType<typeof useGetBookmarksLazyQuery>;
export type GetBookmarksQueryResult = Apollo.QueryResult<GetBookmarksQuery, GetBookmarksQueryVariables>;
export const SearchDocument = gql`
    query search($query: String!, $forceScrape: Boolean!) {
  search(query: $query, forceScrape: $forceScrape) {
    searchHighlightedUrl
    title
    snippet
    url
    host
    bookmarked
    hostaffection
    rank
  }
}
    `;

/**
 * __useSearchQuery__
 *
 * To run a query within a React component, call `useSearchQuery` and pass it any options that fit your needs.
 * When your component renders, `useSearchQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useSearchQuery({
 *   variables: {
 *      query: // value for 'query'
 *      forceScrape: // value for 'forceScrape'
 *   },
 * });
 */
export function useSearchQuery(baseOptions?: ApolloReactHooks.QueryHookOptions<SearchQuery, SearchQueryVariables>) {
        return ApolloReactHooks.useQuery<SearchQuery, SearchQueryVariables>(SearchDocument, baseOptions);
      }
export function useSearchLazyQuery(baseOptions?: ApolloReactHooks.LazyQueryHookOptions<SearchQuery, SearchQueryVariables>) {
          return ApolloReactHooks.useLazyQuery<SearchQuery, SearchQueryVariables>(SearchDocument, baseOptions);
        }
export type SearchQueryHookResult = ReturnType<typeof useSearchQuery>;
export type SearchLazyQueryHookResult = ReturnType<typeof useSearchLazyQuery>;
export type SearchQueryResult = Apollo.QueryResult<SearchQuery, SearchQueryVariables>;
export const GetActivityMonitorDocument = gql`
    query getActivityMonitor {
  getActivityMonitor {
    totalMemory
    usedMemory
    availableMemory
    totalSwap
    usedSwap
  }
}
    `;

/**
 * __useGetActivityMonitorQuery__
 *
 * To run a query within a React component, call `useGetActivityMonitorQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetActivityMonitorQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetActivityMonitorQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetActivityMonitorQuery(baseOptions?: ApolloReactHooks.QueryHookOptions<GetActivityMonitorQuery, GetActivityMonitorQueryVariables>) {
        return ApolloReactHooks.useQuery<GetActivityMonitorQuery, GetActivityMonitorQueryVariables>(GetActivityMonitorDocument, baseOptions);
      }
export function useGetActivityMonitorLazyQuery(baseOptions?: ApolloReactHooks.LazyQueryHookOptions<GetActivityMonitorQuery, GetActivityMonitorQueryVariables>) {
          return ApolloReactHooks.useLazyQuery<GetActivityMonitorQuery, GetActivityMonitorQueryVariables>(GetActivityMonitorDocument, baseOptions);
        }
export type GetActivityMonitorQueryHookResult = ReturnType<typeof useGetActivityMonitorQuery>;
export type GetActivityMonitorLazyQueryHookResult = ReturnType<typeof useGetActivityMonitorLazyQuery>;
export type GetActivityMonitorQueryResult = Apollo.QueryResult<GetActivityMonitorQuery, GetActivityMonitorQueryVariables>;
export const GetCommitHashDocument = gql`
    query getCommitHash {
  getCommitHash
}
    `;

/**
 * __useGetCommitHashQuery__
 *
 * To run a query within a React component, call `useGetCommitHashQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetCommitHashQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetCommitHashQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetCommitHashQuery(baseOptions?: ApolloReactHooks.QueryHookOptions<GetCommitHashQuery, GetCommitHashQueryVariables>) {
        return ApolloReactHooks.useQuery<GetCommitHashQuery, GetCommitHashQueryVariables>(GetCommitHashDocument, baseOptions);
      }
export function useGetCommitHashLazyQuery(baseOptions?: ApolloReactHooks.LazyQueryHookOptions<GetCommitHashQuery, GetCommitHashQueryVariables>) {
          return ApolloReactHooks.useLazyQuery<GetCommitHashQuery, GetCommitHashQueryVariables>(GetCommitHashDocument, baseOptions);
        }
export type GetCommitHashQueryHookResult = ReturnType<typeof useGetCommitHashQuery>;
export type GetCommitHashLazyQueryHookResult = ReturnType<typeof useGetCommitHashLazyQuery>;
export type GetCommitHashQueryResult = Apollo.QueryResult<GetCommitHashQuery, GetCommitHashQueryVariables>;
export const GetRcloneItemsDocument = gql`
    query getRcloneItems($path: String!) {
  getRcloneItems(path: $path) {
    path
    name
    size
    mimeType
    modTime
    isDir
    dirSize
  }
}
    `;

/**
 * __useGetRcloneItemsQuery__
 *
 * To run a query within a React component, call `useGetRcloneItemsQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetRcloneItemsQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetRcloneItemsQuery({
 *   variables: {
 *      path: // value for 'path'
 *   },
 * });
 */
export function useGetRcloneItemsQuery(baseOptions?: ApolloReactHooks.QueryHookOptions<GetRcloneItemsQuery, GetRcloneItemsQueryVariables>) {
        return ApolloReactHooks.useQuery<GetRcloneItemsQuery, GetRcloneItemsQueryVariables>(GetRcloneItemsDocument, baseOptions);
      }
export function useGetRcloneItemsLazyQuery(baseOptions?: ApolloReactHooks.LazyQueryHookOptions<GetRcloneItemsQuery, GetRcloneItemsQueryVariables>) {
          return ApolloReactHooks.useLazyQuery<GetRcloneItemsQuery, GetRcloneItemsQueryVariables>(GetRcloneItemsDocument, baseOptions);
        }
export type GetRcloneItemsQueryHookResult = ReturnType<typeof useGetRcloneItemsQuery>;
export type GetRcloneItemsLazyQueryHookResult = ReturnType<typeof useGetRcloneItemsLazyQuery>;
export type GetRcloneItemsQueryResult = Apollo.QueryResult<GetRcloneItemsQuery, GetRcloneItemsQueryVariables>;
export const SetHostAffectionDocument = gql`
    mutation setHostAffection($host: String!, $affection: Int!) {
  setHostAffection(host: $host, affection: $affection)
}
    `;
export type SetHostAffectionMutationFn = Apollo.MutationFunction<SetHostAffectionMutation, SetHostAffectionMutationVariables>;

/**
 * __useSetHostAffectionMutation__
 *
 * To run a mutation, you first call `useSetHostAffectionMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useSetHostAffectionMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [setHostAffectionMutation, { data, loading, error }] = useSetHostAffectionMutation({
 *   variables: {
 *      host: // value for 'host'
 *      affection: // value for 'affection'
 *   },
 * });
 */
export function useSetHostAffectionMutation(baseOptions?: ApolloReactHooks.MutationHookOptions<SetHostAffectionMutation, SetHostAffectionMutationVariables>) {
        return ApolloReactHooks.useMutation<SetHostAffectionMutation, SetHostAffectionMutationVariables>(SetHostAffectionDocument, baseOptions);
      }
export type SetHostAffectionMutationHookResult = ReturnType<typeof useSetHostAffectionMutation>;
export type SetHostAffectionMutationResult = Apollo.MutationResult<SetHostAffectionMutation>;
export type SetHostAffectionMutationOptions = Apollo.BaseMutationOptions<SetHostAffectionMutation, SetHostAffectionMutationVariables>;
export const SetBookmarkedDocument = gql`
    mutation setBookmarked($url: String!, $bookmarked: Boolean!) {
  setBookmarked(url: $url, bookmarked: $bookmarked)
}
    `;
export type SetBookmarkedMutationFn = Apollo.MutationFunction<SetBookmarkedMutation, SetBookmarkedMutationVariables>;

/**
 * __useSetBookmarkedMutation__
 *
 * To run a mutation, you first call `useSetBookmarkedMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useSetBookmarkedMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [setBookmarkedMutation, { data, loading, error }] = useSetBookmarkedMutation({
 *   variables: {
 *      url: // value for 'url'
 *      bookmarked: // value for 'bookmarked'
 *   },
 * });
 */
export function useSetBookmarkedMutation(baseOptions?: ApolloReactHooks.MutationHookOptions<SetBookmarkedMutation, SetBookmarkedMutationVariables>) {
        return ApolloReactHooks.useMutation<SetBookmarkedMutation, SetBookmarkedMutationVariables>(SetBookmarkedDocument, baseOptions);
      }
export type SetBookmarkedMutationHookResult = ReturnType<typeof useSetBookmarkedMutation>;
export type SetBookmarkedMutationResult = Apollo.MutationResult<SetBookmarkedMutation>;
export type SetBookmarkedMutationOptions = Apollo.BaseMutationOptions<SetBookmarkedMutation, SetBookmarkedMutationVariables>;