mod browse {
    use crate::categories::Category;
    use crate::common::{Pagination, V2BulkResult};
    use crate::media::MediaType;
    use crate::{enum_values, options, Crunchyroll, Locale, MediaCollection, Request, Result};
    use futures_util::FutureExt;
    use serde::Deserialize;

    /// Human readable implementation of [`SimulcastSeason`].
    #[derive(Clone, Debug, Default, Deserialize)]
    #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
    #[cfg_attr(not(feature = "__test_strict"), serde(default))]
    pub struct SimulcastSeasonLocalization {
        pub title: String,
        pub description: String,
    }

    /// A simulcast season.
    #[derive(Clone, Debug, Default, Deserialize, Request)]
    #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
    #[cfg_attr(not(feature = "__test_strict"), serde(default))]
    pub struct SimulcastSeason {
        pub id: String,
        pub localization: SimulcastSeasonLocalization,
    }

    #[allow(dead_code)]
    #[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
    #[request(executor(items))]
    #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
    #[cfg_attr(not(feature = "__test_strict"), serde(default))]
    struct BulkSimulcastSeasonResult {
        items: Vec<SimulcastSeason>,
        total: u32,

        #[cfg(feature = "__test_strict")]
        locale: crate::StrictValue,
    }

    enum_values! {
        pub enum BrowseSortType {
            Popularity = "popularity"
            NewlyAdded = "newly_added"
            Alphabetical = "alphabetical"
        }
    }

    options! {
        BrowseOptions;
        /// Specifies the categories of the entries.
        categories(Vec<Category>, "categories") = None,
        /// Specifies whether the entries should be dubbed.
        is_dubbed(bool, "is_dubbed") = None,
        /// Specifies whether the entries should be subbed.
        is_subbed(bool, "is_subbed") = None,
        /// Specifies a particular simulcast season in which the entries should have been aired. Use
        /// [`Crunchyroll::simulcast_seasons`] to get all seasons.
        simulcast_season(String, "season_tag") = None,
        /// Specifies how the entries should be sorted.
        sort(BrowseSortType, "sort") = Some(BrowseSortType::NewlyAdded),
        /// Specifies the media type of the entries.
        media_type(MediaType, "type") = None,
        /// Preferred audio language.
        preferred_audio_language(Locale, "preferred_audio_language") = None
    }

    impl Crunchyroll {
        /// Browses the crunchyroll catalog filtered by the specified options and returns all found
        /// series and movies.
        pub fn browse(&self, options: BrowseOptions) -> Pagination<MediaCollection> {
            Pagination::new(
                |options| {
                    async move {
                        let endpoint = "https://www.crunchyroll.com/content/v2/discover/browse";
                        let result = options
                            .executor
                            .clone()
                            .get(endpoint)
                            .query(&options.query)
                            .query(&[("n", options.page_size), ("start", options.start)])
                            .request::<V2BulkResult<MediaCollection>>()
                            .await?;
                        Ok((result.data, result.total))
                    }
                    .boxed()
                },
                self.executor.clone(),
                options.into_query(),
            )
        }

        /// Returns all simulcast seasons. The locale specified which language the localization /
        /// human readable name ([`SimulcastSeasonLocalization::title`]) has.
        pub async fn simulcast_seasons(&self, locale: Locale) -> Result<Vec<SimulcastSeason>> {
            let endpoint = "https://www.crunchyroll.com/content/v1/season_list";
            Ok(self
                .executor
                .get(endpoint)
                .query(&[("locale", locale)])
                .request::<BulkSimulcastSeasonResult>()
                .await?
                .items)
        }
    }
}

mod query {
    use crate::common::{Pagination, V2BulkResult, V2TypeBulkResult};
    use crate::media::{Episode, MovieListing, Series};
    use crate::{Crunchyroll, MediaCollection};
    use futures_util::FutureExt;

    /// Results when querying Crunchyroll. Results depending on the input which was given via
    /// [`QueryOptions::result_type`]. If not specified, every field is populated, if one specific
    /// type, for example [`QueryType::Series`], were provided, only [`QueryResults::series`] will
    /// be populated.
    pub struct QueryResults {
        pub top_results: Pagination<MediaCollection>,
        pub series: Pagination<Series>,
        pub movie_listing: Pagination<MovieListing>,
        pub episode: Pagination<Episode>,
    }

    impl Crunchyroll {
        /// Search the Crunchyroll catalog by a given query / string.
        pub fn query<S: AsRef<str>>(&self, query: S) -> QueryResults {
            QueryResults {
                top_results: Pagination::new(
                    |options| {
                        async move {
                            let endpoint = "https://www.crunchyroll.com/content/v2/discover/search";
                            let result: V2BulkResult<V2TypeBulkResult<MediaCollection>> = options
                                .executor
                                .get(endpoint)
                                .query(&options.query)
                                .query(&[("type", "top_results")])
                                .query(&[("limit", options.page_size), ("start", options.start)])
                                .apply_locale_query()
                                .request()
                                .await?;
                            let top_results = result
                                .data
                                .into_iter()
                                .find(|r| r.result_type == "top_results")
                                .unwrap_or_default();
                            Ok((top_results.items, top_results.total))
                        }
                        .boxed()
                    },
                    self.executor.clone(),
                    vec![("q".to_string(), query.as_ref().to_string())],
                ),
                series: Pagination::new(
                    |options| {
                        async move {
                            let endpoint = "https://www.crunchyroll.com/content/v2/discover/search";
                            let result: V2BulkResult<V2TypeBulkResult<Series>> = options
                                .executor
                                .get(endpoint)
                                .query(&options.query)
                                .query(&[("type", "series")])
                                .query(&[("limit", options.page_size), ("start", options.start)])
                                .apply_locale_query()
                                .request()
                                .await?;
                            let top_results = result
                                .data
                                .into_iter()
                                .find(|r| r.result_type == "series")
                                .unwrap_or_default();
                            Ok((top_results.items, top_results.total))
                        }
                        .boxed()
                    },
                    self.executor.clone(),
                    vec![("q".to_string(), query.as_ref().to_string())],
                ),
                movie_listing: Pagination::new(
                    |options| {
                        async move {
                            let endpoint = "https://www.crunchyroll.com/content/v2/discover/search";
                            let result: V2BulkResult<V2TypeBulkResult<MovieListing>> = options
                                .executor
                                .get(endpoint)
                                .query(&options.query)
                                .query(&[("type", "movie_listing")])
                                .query(&[("limit", options.page_size), ("start", options.start)])
                                .apply_locale_query()
                                .request()
                                .await?;
                            let top_results = result
                                .data
                                .into_iter()
                                .find(|r| r.result_type == "movie_listing")
                                .unwrap_or_default();
                            Ok((top_results.items, top_results.total))
                        }
                        .boxed()
                    },
                    self.executor.clone(),
                    vec![("q".to_string(), query.as_ref().to_string())],
                ),
                episode: Pagination::new(
                    |options| {
                        async move {
                            let endpoint = "https://www.crunchyroll.com/content/v2/discover/search";
                            let result: V2BulkResult<V2TypeBulkResult<Episode>> = options
                                .executor
                                .get(endpoint)
                                .query(&options.query)
                                .query(&[("type", "episode")])
                                .query(&[("limit", options.page_size), ("start", options.start)])
                                .apply_locale_query()
                                .request()
                                .await?;
                            let top_results = result
                                .data
                                .into_iter()
                                .find(|r| r.result_type == "episode")
                                .unwrap_or_default();
                            Ok((top_results.items, top_results.total))
                        }
                        .boxed()
                    },
                    self.executor.clone(),
                    vec![("q".to_string(), query.as_ref().to_string())],
                ),
            }
        }
    }
}

pub use browse::*;
pub use query::*;
